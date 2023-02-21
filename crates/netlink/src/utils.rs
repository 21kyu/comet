use std::net::IpAddr;

use anyhow::{bail, Result};

pub fn align_of(len: usize, align_to: usize) -> usize {
    (len + align_to - 1) & !(align_to - 1)
}

pub fn zero_terminated(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    v
}

pub fn vec_to_addr(vec: Vec<u8>) -> Result<IpAddr> {
    // TODO: use IpAddr::parse_ascii when to be stable
    match vec.len() {
        4 => {
            let buf: [u8; 4] = vec.try_into().unwrap();
            Ok(IpAddr::from(buf))
        }
        16 => {
            let buf: [u8; 16] = vec.try_into().unwrap();
            Ok(IpAddr::from(buf))
        }
        _ => {
            bail!("invalid address length: {}", vec.len())
        }
    }
}
