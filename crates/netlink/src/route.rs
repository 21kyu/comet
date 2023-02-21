use std::net::IpAddr;

use anyhow::Result;
use ipnet::IpNet;

use crate::{
    message::{NetlinkRouteAttr, RouteMessage},
    request::NetlinkRequestData,
    utils::vec_to_addr,
};

pub enum RtCmd {
    Add,
    Append,
    Replace,
    Del,
}

#[derive(Default, Debug)]
pub struct Route {
    pub oif_index: i32,
    pub iif_index: i32,
    pub family: u8,
    pub dst: Option<IpNet>,
    pub src: Option<IpAddr>,
    pub gw: Option<IpAddr>,
    pub tos: u8,
    pub table: u8,
    pub protocol: u8,
    pub scope: u8,
    pub rtm_type: u8,
    pub flags: u32,
}

pub fn route_deserialize(buf: &[u8]) -> Result<Route> {
    let if_route_msg = RouteMessage::deserialize(buf)?;
    let rt_attrs = NetlinkRouteAttr::from(&buf[if_route_msg.len()..])?;

    let mut route = Route {
        family: if_route_msg.family,
        tos: if_route_msg.tos,
        table: if_route_msg.table,
        protocol: if_route_msg.protocol,
        scope: if_route_msg.scope,
        rtm_type: if_route_msg.rtm_type,
        ..Default::default()
    };

    for attr in rt_attrs {
        match attr.rt_attr.rta_type {
            libc::RTA_GATEWAY => {
                route.gw = Some(vec_to_addr(attr.value)?);
            }
            libc::RTA_PREFSRC => {
                route.src = Some(vec_to_addr(attr.value)?);
            }
            libc::RTA_DST => {
                route.dst = Some(IpNet::new(vec_to_addr(attr.value)?, if_route_msg.dst_len)?);
            }
            libc::RTA_OIF => {
                route.oif_index = i32::from_ne_bytes(attr.value[..4].try_into()?);
            }
            libc::RTA_IIF => {
                route.iif_index = i32::from_ne_bytes(attr.value[..4].try_into()?);
            }
            // TODO: more types
            _ => {}
        }
    }

    Ok(route)
}
