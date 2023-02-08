use anyhow::{bail, Result};
use network::{ip::nmap, netlink::get_ip_addr, run_command};
use std::{
    fs::{self, File},
    io::{self, BufRead, Write},
    net::Ipv4Addr,
    path::Path,
};

pub const IP_STORE: &str = "/tmp/reserved_ips";

fn read_lines<P>(path: P) -> Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_reserved_ips(path: &str) -> Result<Vec<Ipv4Addr>> {
    if let Ok(lines) = read_lines(path) {
        return Ok(lines
            .map(|ip| ip.unwrap().parse::<Ipv4Addr>().unwrap())
            .collect::<Vec<_>>());
    }

    bail!("Failed to get reserved IPs")
}

pub fn allocate_ip(subnet: &str, ip_store_path: &str) -> Result<(Ipv4Addr, Ipv4Addr, Ipv4Addr)> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(ip_store_path)
        .unwrap();

    let mut all_ips = nmap(subnet)?;
    let reserved_ips = get_reserved_ips(ip_store_path)?;

    for ip in reserved_ips {
        all_ips.remove(&ip);
    }

    let skip_ip = all_ips.pop_first().unwrap();
    let gw_ip = all_ips.pop_first().unwrap();
    let cont_ip = all_ips.pop_first().unwrap();

    writeln!(file, "{}", &cont_ip.to_string()).unwrap();

    Ok((skip_ip, gw_ip, cont_ip))
}

pub fn release_ip(if_name: &str, ip_store_path: &str) -> Result<()> {
    let ip_addr = get_ip_addr(if_name)?;
    let opt = format!("/{ip_addr}/d");
    let out = run_command!("sed", "-i", &opt, ip_store_path);

    if let false = out.status.success() {
        bail!("Failed to release ip")
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use network::{run_command, test_setup};

    use crate::ipam::allocator::release_ip;

    use super::{allocate_ip, get_reserved_ips};

    #[test]
    fn get_reserved_ips_test() {
        let ip_store_path = "/tmp/reserved_ips_test";

        fs::write(ip_store_path, "10.244.0.2\n10.244.0.3\n10.244.0.4\n").unwrap();

        let reserved_ips = get_reserved_ips(ip_store_path).unwrap();

        assert_eq!(reserved_ips.len(), 3);

        fs::remove_file(ip_store_path).unwrap();
    }

    #[test]
    fn allocate_ip_test() {
        let subnet = "10.244.0.0/24";
        let ip_store_path = "/tmp/allocate_ip_test";

        fs::write(ip_store_path, "10.244.0.2\n10.244.0.3\n10.244.0.4\n").unwrap();

        let (skip_ip, gw_ip, cont_ip) = allocate_ip(subnet, ip_store_path).unwrap();

        assert_eq!(skip_ip.to_string(), "10.244.0.0");
        assert_eq!(gw_ip.to_string(), "10.244.0.1");
        assert_eq!(cont_ip.to_string(), "10.244.0.5");

        let reserved_ips = get_reserved_ips(ip_store_path).unwrap();

        assert_eq!(reserved_ips.len(), 4);
        assert_eq!(reserved_ips.last().unwrap().to_string(), "10.244.0.5");

        fs::remove_file(ip_store_path).unwrap();
    }

    #[test]
    fn release_ip_test() {
        test_setup!();
        let if_name = "cni0";
        let ip_store_path = "/tmp/release_ip_test";

        fs::write(ip_store_path, "10.244.0.0\n10.244.0.1\n10.244.0.2\n").unwrap();

        run_command!("ip", "link", "add", if_name, "type", "bridge");
        run_command!("ip", "link", "set", if_name, "up");
        run_command!("ip", "addr", "add", "10.244.0.1/24", "dev", if_name);

        release_ip(if_name, ip_store_path).unwrap();

        let reserved_ips = get_reserved_ips(ip_store_path).unwrap();

        println!("{reserved_ips:?}");
        assert_eq!(reserved_ips.len(), 2);
    }
}
