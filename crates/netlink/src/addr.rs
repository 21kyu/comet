use std::net::IpAddr;

use anyhow::Result;
use ipnet::IpNet;

use crate::{
    message::{AddressMessage, NetlinkRouteAttr},
    request::NetlinkRequestData,
    utils::vec_to_addr,
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
    pub flags: u8,
    pub scope: u8,
    pub broadcast: Option<IpAddr>,
    pub peer: Option<IpNet>,
    pub preferred_lifetime: i32,
    pub valid_lifetime: i32,
}

pub fn addr_deserialize(buf: &[u8]) -> Result<Address> {
    let if_addr_msg = AddressMessage::deserialize(buf)?;
    let rt_attrs = NetlinkRouteAttr::from(&buf[if_addr_msg.len()..])?;

    let mut addr = Address {
        index: if_addr_msg.index,
        scope: if_addr_msg.scope,
        ..Default::default()
    };

    for attr in rt_attrs {
        match attr.rt_attr.rta_type {
            libc::IFA_ADDRESS => {
                addr.ip = IpNet::new(vec_to_addr(attr.value)?, if_addr_msg.prefix_len)?;
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
