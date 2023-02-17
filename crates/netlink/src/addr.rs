use std::net::IpAddr;

use anyhow::{bail, Result};
use ipnet::IpNet;

use crate::{
    message::{IfAddrMessage, NetlinkRouteAttr},
    request::NetlinkRequestData,
};

pub enum AddrCmd {
    Add,
    Change,
    Replace,
    Del,
}

#[derive(Default, Debug)]
pub struct Address {
    pub index: i32,
    pub ip: IpNet,
    pub label: String,
    pub flags: i32,
    pub scope: i32,
    pub broadcast: Option<IpAddr>,
    pub peer: Option<IpNet>,
    pub preferred_lifetime: i32,
    pub valid_lifetime: i32,
}

pub fn addr_deserialize(buf: &[u8]) -> Result<Address> {
    let if_addr_msg = IfAddrMessage::deserialize(buf)?;
    let rt_attrs = NetlinkRouteAttr::from(&buf[if_addr_msg.len()..])?;

    let mut addr = Address {
        index: if_addr_msg.ifa_index,
        scope: if_addr_msg.ifa_scope as i32,
        ..Default::default()
    };

    for attr in rt_attrs {
        match attr.rt_attr.rta_type {
            libc::IFA_ADDRESS => {
                addr.ip = IpNet::new(vec_to_addr(attr.value)?, if_addr_msg.ifa_prefix_len)?;
            }
            libc::IFA_LOCAL => {
                // TODO
            }
            libc::IFA_BROADCAST => {
                // TODO
            }
            libc::IFA_LABEL => {
                // TODO
            }
            libc::IFA_CACHEINFO => {
                // TODO
            }
            _ => {}
        }
    }

    Ok(addr)
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
