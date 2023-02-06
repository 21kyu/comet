use std::collections::HashMap;

use anyhow::Result;

use crate::{
    consts,
    request::NetlinkRequestData,
    socket::{IfInfoMessage, NetlinkRouteAttr},
};

pub enum Namespace {
    Pid(i32),
    Fd(i32),
}

pub enum Kind {
    Device(LinkAttrs),
    Dummy(LinkAttrs),
    Bridge {
        attrs: LinkAttrs,
        hello_time: Option<u32>,
        ageing_time: Option<u32>,
        multicast_snooping: Option<bool>,
        vlan_filtering: Option<bool>,
    },
    Veth {
        attrs: LinkAttrs,
        peer_name: String,
        peer_hw_addr: Vec<u8>,
        peer_ns: Option<Namespace>,
    },
}

pub trait Link {
    fn link_type(&self) -> String;
    fn attrs(&self) -> &LinkAttrs;
    fn kind(&self) -> &Kind;
}

#[derive(Debug, Default, Clone)]
pub struct LinkAttrs {
    pub link_type: String,
    pub index: i32,
    pub name: String,
    hw_addr: Vec<u8>,
    pub mtu: u32,
    pub flags: u32,
    raw_flags: u32,
    parent_index: i32,
    master_index: i32,
    pub tx_queue_len: i32,
    alias: String,
    xdp: LinkXdp,
    prot_info: String,
    oper_state: u8,
    phys_switch_id: i32,
    netns_id: i32,
    gso_max_size: u32,
    gso_max_segs: u32,
    gro_max_size: u32,
    vfs: String,
    pub num_tx_queues: i32,
    pub num_rx_queues: i32,
    group: u32,
    statistics: String,
}

impl LinkAttrs {
    pub fn new() -> Self {
        Self::default()
    }

    fn from(if_info_msg: IfInfoMessage) -> Self {
        let mut attrs = Self::new();
        attrs.index = if_info_msg.ifi_index;
        attrs.raw_flags = if_info_msg.ifi_flags;
        attrs
    }
}

impl Link for Kind {
    fn link_type(&self) -> String {
        match self {
            Kind::Device(_) => "device".to_string(),
            Kind::Dummy(_) => "dummy".to_string(),
            Kind::Bridge { .. } => "bridge".to_string(),
            Kind::Veth { .. } => "veth".to_string(),
        }
    }

    fn attrs(&self) -> &LinkAttrs {
        match self {
            Kind::Device(attrs) => attrs,
            Kind::Dummy(attrs) => attrs,
            Kind::Bridge { attrs, .. } => attrs,
            Kind::Veth { attrs, .. } => attrs,
        }
    }

    fn kind(&self) -> &Kind {
        self
    }
}

#[derive(Debug, Default, Clone)]
struct LinkXdp {
    fd: i32,
    attached: bool,
    attache_mode: u32,
    flags: u32,
    prog_id: u32,
}

impl LinkXdp {
    fn new() -> Self {
        Self::default()
    }

    fn parse(data: &[u8]) -> Self {
        let mut xdp = Self::new();

        let rt_attrs = NetlinkRouteAttr::from(data).unwrap();
        for attr in rt_attrs {
            match attr.rt_attr.rta_type {
                consts::IFLA_XDP_FD => {
                    xdp.fd = i32::from_ne_bytes(attr.value[..4].try_into().unwrap());
                }
                consts::IFLA_XDP_ATTACHED => {
                    xdp.attache_mode = attr.value[0].try_into().unwrap();
                    xdp.attached = attr.value[0] != 0;
                }
                consts::IFLA_XDP_FLAGS => {
                    xdp.flags = u32::from_ne_bytes(attr.value[..4].try_into().unwrap());
                }
                consts::IFLA_XDP_PROG_ID => {
                    xdp.prog_id = u32::from_ne_bytes(attr.value[..4].try_into().unwrap());
                }
                _ => {
                    println!("--> Unknown rt_attr.rta_type: {}", attr.rt_attr.rta_type);
                }
            }
        }

        xdp
    }
}

pub fn link_deserialize(buf: &[u8]) -> Result<Box<dyn Link>> {
    let if_info_msg = IfInfoMessage::deserialize(buf)?;
    let rt_attrs = NetlinkRouteAttr::from(&buf[if_info_msg.len()..])?;

    let mut base = LinkAttrs::from(if_info_msg);
    let mut data = HashMap::new();

    for attr in rt_attrs {
        match attr.rt_attr.rta_type {
            libc::IFLA_LINKINFO => {
                data = extract_link_info(&mut base, NetlinkRouteAttr::from(&attr.value)?)?
            }
            libc::IFLA_ADDRESS => {
                base.hw_addr = attr.value;
                println!("IFLA_ADDRESS: {:02x?}", base.hw_addr);
            }
            libc::IFLA_IFNAME => {
                base.name = String::from_utf8(attr.value[..attr.value.len() - 1].to_vec())?;
                println!("IFLA_IFNAME: {}", base.name);
            }
            libc::IFLA_MTU => {
                base.mtu = u32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_MTU: {}", base.mtu);
            }
            libc::IFLA_LINK => {
                base.parent_index = i32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_LINK: {}", base.parent_index);
            }
            libc::IFLA_MASTER => {
                base.master_index = i32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_MASTER: {}", base.master_index);
            }
            libc::IFLA_TXQLEN => {
                base.tx_queue_len = i32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_TXQLEN: {}", base.tx_queue_len);
            }
            libc::IFLA_IFALIAS => {
                base.alias = String::from_utf8(attr.value[..attr.value.len() - 1].to_vec())?;
                println!("IFLA_IFALIAS: {}", base.alias);
            }
            libc::IFLA_STATS => {
                // TODO
                println!("IFLA_STATS: TODO");
            }
            libc::IFLA_STATS64 => {
                // TODO
                println!("IFLA_STATS64: TODO");
            }
            libc::IFLA_XDP => {
                base.xdp = LinkXdp::parse(&attr.value);
                println!("IFLA_XDP: {:?}", base.xdp);
            }
            libc::IFLA_PROTINFO | consts::NLA_F_NESTED => {
                // TODO
                println!("IFLA_PROTINFO: TODO");
            }
            libc::IFLA_OPERSTATE => {
                base.oper_state = attr.value[0];
                println!("IFLA_OPERSTATE: {}", base.oper_state);
            }
            libc::IFLA_PHYS_SWITCH_ID => {
                base.phys_switch_id = i32::from_be_bytes(attr.value[..4].try_into()?);
                println!("IFLA_PHYS_SWITCH_ID: {:?}", base.phys_switch_id);
            }
            libc::IFLA_LINK_NETNSID => {
                base.netns_id = i32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_LINK_NETNSID: {:?}", base.netns_id);
            }
            libc::IFLA_GSO_MAX_SIZE => {
                base.gso_max_size = u32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_GSO_MAX_SIZE: {:?}", base.gso_max_size);
            }
            libc::IFLA_GSO_MAX_SEGS => {
                base.gso_max_segs = u32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_GSO_MAX_SEGS: {:?}", base.gso_max_segs);
            }
            consts::IFLA_GRO_MAX_SIZE => {
                base.gro_max_size = u32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_GRO_MAX_SIZE: {:?}", base.gro_max_size);
            }
            libc::IFLA_VFINFO_LIST => {
                // TODO
                println!("IFLA_VFINFO_LIST: TODO");
            }
            libc::IFLA_NUM_TX_QUEUES => {
                base.num_tx_queues = i32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_NUM_TX_QUEUES: {:?}", base.num_tx_queues);
            }
            libc::IFLA_NUM_RX_QUEUES => {
                base.num_rx_queues = i32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_NUM_RX_QUEUES: {:?}", base.num_rx_queues);
            }
            libc::IFLA_GROUP => {
                base.group = u32::from_ne_bytes(attr.value[..4].try_into()?);
                println!("IFLA_GROUP: {:?}", base.group);
            }
            _ => {
                println!("Unknown attribute: {}", attr.rt_attr.rta_type);
            }
        }
    }

    println!("index: {:#?}", base);

    Ok(match &base.link_type[..] {
        "dummy" => Box::new(Kind::Dummy(base)),
        "bridge" => Box::new(Kind::Bridge {
            attrs: base,
            hello_time: data
                .get(&consts::IFLA_BR_HELLO_TIME)
                .map(|v| u32::from_ne_bytes(v[..4].try_into().unwrap())),
            ageing_time: data
                .get(&consts::IFLA_BR_AGEING_TIME)
                .map(|v| u32::from_ne_bytes(v[..4].try_into().unwrap())),
            multicast_snooping: data.get(&consts::IFLA_BR_MCAST_SNOOPING).map(|v| v[0] == 1),
            vlan_filtering: data.get(&consts::IFLA_BR_VLAN_FILTERING).map(|v| v[0] == 1),
        }),
        "veth" => Box::new(Kind::Veth {
            // TODO: need to parse peer info..?
            attrs: base,
            peer_name: "".to_string(),
            peer_hw_addr: vec![],
            peer_ns: None,
        }),
        "device" | _ => Box::new(Kind::Device(base)),
    })
}

fn extract_link_info(
    base: &mut LinkAttrs,
    infos: Vec<NetlinkRouteAttr>,
) -> Result<HashMap<u16, Vec<u8>>> {
    let mut data = HashMap::new();

    for info in infos {
        match info.rt_attr.rta_type {
            libc::IFLA_INFO_KIND => {
                base.link_type = String::from_utf8(info.value[..info.value.len() - 1].to_vec())?;
                println!("IFLA_INFO_KIND: {}", base.link_type);
            }
            libc::IFLA_INFO_DATA => {
                data = NetlinkRouteAttr::map(&info.value)?;
                println!("IFLA_INFO_DATA");
            }
            libc::IFLA_INFO_SLAVE_KIND => {
                // TODO
                println!("IFLA_INFO_SLAVE_KIND: TODO");
            }
            libc::IFLA_INFO_SLAVE_DATA => {
                // TODO
                println!("IFLA_INFO_SLAVE_DATA: TODO");
            }
            _ => {
                println!("-> Unknown attribute: {}", info.rt_attr.rta_type);
            }
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    static NETLINK_MSG: [u8; 1752] = [
        0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x03, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x0C, 0x00, 0x03, 0x00, 0x64, 0x6F, 0x63, 0x6B, 0x65, 0x72, 0x30, 0x00, 0x08, 0x00,
        0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x10, 0x00, 0x02, 0x00, 0x00, 0x00, 0x05,
        0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0xDC, 0x05, 0x00, 0x00,
        0x08, 0x00, 0x32, 0x00, 0x44, 0x00, 0x00, 0x00, 0x08, 0x00, 0x33, 0x00, 0xFF, 0xFF, 0x00,
        0x00, 0x08, 0x00, 0x1B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x1E, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x08, 0x00, 0x1F, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x28, 0x00, 0xFF,
        0xFF, 0x00, 0x00, 0x08, 0x00, 0x29, 0x00, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x20, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x06,
        0x00, 0x6E, 0x6F, 0x71, 0x75, 0x65, 0x75, 0x65, 0x00, 0x08, 0x00, 0x23, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x08, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x30, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x05, 0x00, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00, 0x0E, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x0A, 0x00, 0x01, 0x00, 0x02, 0x42, 0x3B, 0x14, 0xA7, 0x98, 0x00, 0x00, 0x0A,
        0x00, 0x02, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xC4, 0x00, 0x17, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x07,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x2B, 0x00, 0x05, 0x00, 0x02, 0x00,
        0x00, 0x00, 0x00, 0x00, 0xAC, 0x01, 0x12, 0x00, 0x0B, 0x00, 0x01, 0x00, 0x62, 0x72, 0x69,
        0x64, 0x67, 0x65, 0x00, 0x00, 0x9C, 0x01, 0x02, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x0C, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x0C, 0x00, 0x13, 0x00, 0x71, 0x16, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01,
        0x00, 0xDC, 0x05, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0xC8, 0x00, 0x00, 0x00, 0x08, 0x00,
        0x03, 0x00, 0xD0, 0x07, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x30, 0x75, 0x00, 0x00, 0x08,
        0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x06, 0x00, 0x00, 0x80, 0x00, 0x00,
        0x05, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x0C, 0x00, 0x0B, 0x00, 0x80, 0x00, 0x02, 0x42, 0x3B, 0x14, 0xA7, 0x98, 0x0C, 0x00,
        0x0A, 0x00, 0x80, 0x00, 0x02, 0x42, 0x3B, 0x14, 0xA7, 0x98, 0x06, 0x00, 0x0C, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x08, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x0E, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x14,
        0x00, 0x01, 0x80, 0xC2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x2E, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x06, 0x00, 0x08, 0x00, 0x81, 0x00, 0x00, 0x00, 0x06,
        0x00, 0x27, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x05, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x16, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x05, 0x00, 0x17, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x18, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x05, 0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x2A, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x08, 0x00, 0x1A, 0x00, 0x10, 0x00, 0x00, 0x00, 0x08, 0x00, 0x1B, 0x00,
        0x00, 0x10, 0x00, 0x00, 0x08, 0x00, 0x1C, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x1D,
        0x00, 0x02, 0x00, 0x00, 0x00, 0x05, 0x00, 0x2B, 0x00, 0x02, 0x00, 0x00, 0x00, 0x05, 0x00,
        0x2C, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x1E, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x0C, 0x00, 0x1F, 0x00, 0x90, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x0C, 0x00, 0x20, 0x00, 0x9C, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x21,
        0x00, 0xD4, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x22, 0x00, 0xE8, 0x03,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x23, 0x00, 0x34, 0x0C, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x05, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x25, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x26, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x03, 0x1A,
        0x00, 0x88, 0x00, 0x02, 0x00, 0x84, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x27, 0x00, 0x00, 0xE8, 0x03, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x80, 0x02, 0x0A, 0x00, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x05, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xC2, 0xC5, 0x77, 0x00, 0x0C, 0x89, 0x00, 0x00,
        0xE8, 0x03, 0x00, 0x00, 0xE4, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00,
        0x00, 0xDC, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xA0, 0x0F, 0x00, 0x00, 0xE8,
        0x03, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x80, 0x3A, 0x09, 0x00, 0x80, 0x51, 0x01, 0x00,
        0x03, 0x00, 0x00, 0x00, 0x58, 0x02, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x60, 0xEA,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x27, 0x00, 0x00, 0xE8, 0x03, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80, 0xEE, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF,
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x2C, 0x01, 0x03, 0x00, 0x25, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x00, 0x06, 0x00, 0x06, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_link_deserialize() {
        let link = link_deserialize(&NETLINK_MSG).unwrap();
        assert_eq!(link.link_type(), "bridge");

        let attrs = link.attrs();
        assert_eq!(attrs.index, 4);
        assert_eq!(attrs.name, "docker0");
        assert_eq!(attrs.mtu, 1500);
        assert_eq!(attrs.raw_flags, 0x1003);

        match link.kind() {
            Kind::Bridge {
                attrs: _,
                hello_time,
                ageing_time,
                multicast_snooping,
                vlan_filtering,
            } => {
                assert_eq!(hello_time.unwrap(), 200);
                assert_eq!(ageing_time.unwrap(), 30000);
                assert_eq!(multicast_snooping.unwrap(), true);
                assert_eq!(vlan_filtering.unwrap(), false);
            }
            _ => panic!("Expected bridge link"),
        }
    }
}
