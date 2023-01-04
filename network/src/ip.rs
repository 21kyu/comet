use anyhow::Result;
use std::{collections::BTreeSet, net::Ipv4Addr};

fn ip_to_binary(ip: Ipv4Addr) -> u32 {
    let octets = ip.octets();
    (octets[0] as u32) << 24
        | (octets[1] as u32) << 16
        | (octets[2] as u32) << 8
        | (octets[3] as u32)
}

pub fn nmap(subnet: &str) -> Result<BTreeSet<Ipv4Addr>> {
    let (ip, subnet_mask_size) = subnet.split_once('/').unwrap();
    let ip = ip.parse::<Ipv4Addr>()?;
    let subnet_mask_size = subnet_mask_size.parse::<u8>()?;
    let subnet_mask = Ipv4Addr::from(u32::max_value() << (32 - subnet_mask_size));

    let ip = ip_to_binary(ip) & ip_to_binary(subnet_mask);
    let mut ips = BTreeSet::new();

    for i in 0..(1 << (32 - subnet_mask_size)) {
        ips.insert(Ipv4Addr::from(ip + i));
    }

    Ok(ips)
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::ip::{ip_to_binary, nmap};

    #[test]
    fn ip_to_binary_test() {
        let ip = Ipv4Addr::new(10, 244, 0, 1);
        let binary = ip_to_binary(ip);

        assert_eq!(binary, 0b00001010_11110100_00000000_00000001);
    }

    #[test]
    fn nmap_test() {
        let subnet = "10.244.0.1/24";
        let ips = nmap(subnet).unwrap();

        assert_eq!(ips.len(), 256);
    }
}
