use anyhow::{bail, Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::net::Ipv4Addr;

#[macro_export]
macro_rules! run_command {
    ($command:expr $(, $args:expr)*) => {
        std::process::Command::new($command).args([$($args),*]).output()
            .expect("failed to run command")
    };
}

pub fn add_link(if_name: &str, link_type: &str) -> Result<()> {
    let out = run_command!("ip", "link", "add", if_name, "type", link_type);

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn create_veth_pair(host_if_name: &str, peer_if_name: &str) -> Result<()> {
    let out = run_command!(
        "ip",
        "link",
        "add",
        host_if_name,
        "type",
        "veth",
        "peer",
        "name",
        peer_if_name
    );

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn set_up(if_name: &str) -> Result<()> {
    let out = run_command!("ip", "link", "set", if_name, "up");

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn set_master(host_if_name: &str, bridge_if_name: &str) -> Result<()> {
    let out = run_command!("ip", "link", "set", host_if_name, "master", bridge_if_name);

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn set_netns(peer_if_name: &str, netns: &str) -> Result<()> {
    let out = run_command!("ip", "link", "set", peer_if_name, "netns", netns);

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn set_link_name(peer_if_name: &str, cont_if_name: &str) -> Result<()> {
    let out = run_command!("ip", "link", "set", peer_if_name, "name", cont_if_name);

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn add_addr(cont_ip: Ipv4Addr, subnet_mask_size: &str, if_name: &str) -> Result<()> {
    let ip = format!("{}/{}", cont_ip, subnet_mask_size);
    let out = run_command!("ip", "addr", "add", &ip, "dev", if_name);

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn add_default_route(gw_ip: Ipv4Addr, if_name: &str) -> Result<()> {
    let out = run_command!(
        "ip",
        "route",
        "add",
        "default",
        "via",
        &gw_ip.to_string(),
        "dev",
        if_name
    );

    match out.status.success() {
        true => Ok(()),
        _ => Err(Error::msg(String::from_utf8(out.stderr).unwrap())),
    }
}

pub fn get_mac_addr(if_name: &str) -> Result<String> {
    let out = run_command!("ip", "link", "show", if_name);

    if let true = out.status.success() {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"/ether (?P<mac>[\w:]+) .*").unwrap();
        }

        return Ok(RE
            .captures(&String::from_utf8(out.stdout)?)
            .and_then(|cap| cap.name("mac").map(|mac| mac.as_str().to_string()))
            .unwrap());
    }

    bail!("Failed to get mac address")
}

pub fn get_ip_addr(if_name: &str) -> Result<String> {
    let out = run_command!("ip", "addr", "show", if_name);

    if let true = out.status.success() {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"inet (?P<ip>(\b25[0-5]|\b2[0-4][0-9]|\b[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3})/").unwrap();
        }

        return Ok(RE
            .captures(&String::from_utf8(out.stdout)?)
            .and_then(|cap| cap.name("ip").map(|ip| ip.as_str().to_string()))
            .unwrap());
    }

    bail!("Failed to get ip address")
}

#[cfg(test)]
mod tests {
    use crate::netlink::{get_ip_addr, get_mac_addr};

    #[test]
    fn get_mac_addr_test() {
        let if_name = "cni0";

        run_command!("ip", "link", "add", if_name, "type", "bridge");
        run_command!("ip", "link", "set", if_name, "up");
        run_command!("ip", "addr", "add", "10.244.0.1/24", "dev", if_name);

        let mac = get_mac_addr(if_name).unwrap();
        assert!(!mac.is_empty());
    }

    #[test]
    fn get_ip_addr_test() {
        let if_name = "cni0";

        run_command!("ip", "link", "add", if_name, "type", "bridge");
        run_command!("ip", "link", "set", if_name, "up");
        run_command!("ip", "addr", "add", "10.244.0.1/24", "dev", if_name);

        let ip = get_ip_addr(if_name).unwrap();
        assert_eq!(ip, "10.244.0.1");
    }
}
