use std::fs::File;
use std::os::fd::AsRawFd;
use std::thread;

use anyhow::Result;
use nix::sched;

use crate::{
    ipam::{self, allocator::release_ip},
    network::netlink::{
        add_addr, add_default_route, create_veth_pair, get_mac_addr, set_link_name, set_master,
        set_netns, set_up,
    },
};

fn create_if_name(prefix: &str, cont_id: &str) -> Result<String> {
    Ok(match cont_id.len() < 5 {
        true => format!("{}{}", prefix, cont_id),
        false => format!("{}{}", prefix, &cont_id[..5]),
    })
}

pub fn setup_veth(
    br_if_name: &str,
    cni_if_name: &str,
    cont_id: &str,
    subnet: &str,
    netns_path: &str,
) -> Result<(String, String, String)> {
    let veth_if_name = create_if_name("veth", cont_id)?;
    let peer_if_name = create_if_name("peer", cont_id)?;
    let netns_name = netns_path.split("/").last().unwrap();

    create_veth_pair(&veth_if_name, &peer_if_name)?;
    set_up(&veth_if_name)?;
    set_master(&veth_if_name, br_if_name)?;
    set_netns(&peer_if_name, netns_name)?;

    let (_, gw_ip, cont_ip) = ipam::allocator::allocate_ip(subnet, ipam::allocator::IP_STORE)?;

    let netns_file = File::open(netns_path)?;
    let netns_fd = netns_file.as_raw_fd();
    let subnet_mask_size = subnet.split("/").last().unwrap().to_string();
    let if_name = cni_if_name.to_string();
    let address = format!("{}/{}", cont_ip, subnet_mask_size);

    let handle = thread::spawn(move || -> Result<String> {
        sched::setns(netns_fd, sched::CloneFlags::CLONE_NEWNET)?;
        set_link_name(&peer_if_name, &if_name)?;
        set_up(&if_name)?;
        add_addr(cont_ip, &subnet_mask_size, &if_name)?;
        add_default_route(gw_ip, &if_name)?;
        get_mac_addr(&if_name)
    });

    let mac = handle.join().unwrap().unwrap();

    Ok((mac, address, gw_ip.to_string()))
}

pub fn release_veth(if_name: &str, netns: &str) -> Result<()> {
    let netns_file = File::open(netns)?;
    let netns_fd = netns_file.as_raw_fd();

    sched::setns(netns_fd, sched::CloneFlags::CLONE_NEWNET)?;
    release_ip(if_name, ipam::allocator::IP_STORE)
}

#[cfg(test)]
mod tests {
    use crate::connector::veth::{create_if_name, setup_veth};
    use crate::run_command;

    #[test]
    fn veth_test() {
        let br_if_name = "cni0";
        let cni_if_name = "eth0";
        let cont_id = "123456789";
        let subnet = "10.244.0.0/24";
        let netns_path = &format!("/var/run/netns/{}", cont_id);

        run_command!("ip", "link", "add", br_if_name, "type", "bridge");
        run_command!("ip", "link", "set", br_if_name, "up");
        run_command!("ip", "addr", "add", "10.244.0.1/24", "dev", br_if_name);
        run_command!("ip", "netns", "add", cont_id);

        let (mac, address, gw_ip) =
            setup_veth(br_if_name, cni_if_name, cont_id, subnet, netns_path).unwrap();

        assert!(!mac.is_empty());
        assert!(!address.is_empty());
        assert_eq!(gw_ip, "10.244.0.1");

        let veth_if_name = &create_if_name("veth", cont_id).unwrap();
        let out = run_command!("ip", "link", "del", veth_if_name);

        assert!(out.status.success(), "Failed to delete veth pair")
    }
}
