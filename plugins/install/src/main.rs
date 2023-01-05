use std::{fs, net::Ipv4Addr};

use anyhow::Result;
use network::netlink::{add_addr, add_link, set_up};

const BRIDGE_IF_NAME: &str = "cni0";
const CNI_CONF_PATH: &str = "/etc/cni/net.d/08-comet.conf";

fn setup_bridge(bridge_ip: Ipv4Addr, subnet_mask_size: &str) -> Result<()> {
    add_link(BRIDGE_IF_NAME, "bridge")?;
    set_up(BRIDGE_IF_NAME)?;
    add_addr(bridge_ip, subnet_mask_size, BRIDGE_IF_NAME)?;

    Ok(())
}

fn main() {
    let bridge_ip = Ipv4Addr::new(10, 244, 0, 1);
    let subnet_mask_size = "24";

    setup_bridge(bridge_ip, subnet_mask_size).unwrap();

    let net_conf = r#"{
        "cniVersion": "0.3.1",
        "name": "comet",
        "type": "comet-cni",
        "network": "10.244.0.0/16",
        "subnet": "10.244.0.0/24"
    }"#;

    fs::write(CNI_CONF_PATH, net_conf).unwrap();
}
