use anyhow::Result;
use serde::Serialize;

use crate::connector::veth::setup_veth;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Response<'a> {
    cni_version: &'a str,
    interfaces: Vec<Interface<'a>>,
    ips: Vec<Ip<'a>>,
}

impl<'a> Response<'a> {
    pub fn new(interfaces: Vec<Interface<'a>>, ips: Vec<Ip<'a>>) -> Self {
        Self {
            cni_version: "0.3.0",
            interfaces,
            ips,
        }
    }
}

#[derive(Debug, Serialize)]
struct Interface<'a> {
    name: &'a str,
    mac: &'a str,
    sandbox: &'a str,
}

impl<'a> Interface<'a> {
    pub fn new(name: &'a str, mac: &'a str, sandbox: &'a str) -> Self {
        Self { name, mac, sandbox }
    }
}

#[derive(Debug, Serialize)]
struct Ip<'a> {
    version: &'a str,
    address: &'a str,
    gateway: &'a str,
    interface: u8,
}

impl<'a> Ip<'a> {
    pub fn new(address: &'a str, gateway: &'a str) -> Self {
        Self {
            version: "4",
            address,
            gateway,
            interface: 0,
        }
    }
}

pub fn add(cni_if_name: &str, container_id: &str, subnet: &str, netns: &str) -> Result<String> {
    let br_if_name = "cni0";

    let (mac, address, gateway) = setup_veth(br_if_name, cni_if_name, container_id, subnet, netns)?;

    let interface = Interface::new(cni_if_name, &mac, netns);
    let ip = Ip::new(&address, &gateway);
    let res = Response::new(vec![interface], vec![ip]);

    Ok(serde_json::to_string(&res)?)
}

#[cfg(test)]
mod tests {
    use network::{run_command, test_setup};

    use crate::command::add::add;

    #[test]
    fn add_test() {
        test_setup!();
        let cni_if_name = "eth0";
        let container_id = "123456789";
        let subnet = "10.244.0.0/24";
        let netns = &format!("/var/run/netns/{container_id}");

        run_command!("ip", "link", "add", "cni0", "type", "bridge");
        run_command!("ip", "link", "set", "cni0", "up");
        run_command!("ip", "addr", "add", "10.244.0.1/24", "dev", "cni0");
        run_command!("ip", "netns", "add", container_id);

        let res = add(cni_if_name, container_id, subnet, netns).unwrap();

        println!("{res}");
        assert!(!res.is_empty());

        let out = run_command!("ip", "link", "del", "veth12345");

        assert!(out.status.success(), "Failed to delete veth pair")
    }
}
