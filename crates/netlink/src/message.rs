use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use crate::{consts, request::NetlinkRequestData, utils::align_of};

pub struct NetlinkMessage {
    pub header: NetlinkMessageHeader,
    pub data: Vec<u8>,
}

impl NetlinkMessage {
    pub fn from(mut buf: &[u8]) -> std::io::Result<Vec<Self>> {
        let mut msgs = Vec::new();

        while buf.len() >= consts::NLMSG_HDRLEN {
            let header = unsafe { *(buf.as_ptr() as *const NetlinkMessageHeader) };
            let len = align_of(header.nlmsg_len as usize, consts::NLMSG_ALIGNTO);
            let data = buf[consts::NLMSG_HDRLEN..header.nlmsg_len as usize].to_vec();

            msgs.push(Self { header, data });
            buf = &buf[len..];
        }

        Ok(msgs)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Serialize, Debug)]
pub struct NetlinkMessageHeader {
    pub nlmsg_len: u32,
    pub nlmsg_type: u16,
    pub nlmsg_flags: u16,
    pub nlmsg_seq: u32,
    pub nlmsg_pid: u32,
}

impl NetlinkMessageHeader {
    pub fn new(proto: u16, flags: i32) -> Self {
        Self {
            nlmsg_len: std::mem::size_of::<Self>() as u32,
            nlmsg_type: proto,
            nlmsg_flags: (libc::NLM_F_REQUEST | flags) as u16,
            nlmsg_seq: 0,
            nlmsg_pid: 0,
        }
    }
}

pub struct NetlinkRouteAttr {
    pub rt_attr: RtAttr,
    pub value: Vec<u8>,
    pub children: Option<Vec<Box<dyn NetlinkRequestData>>>,
}

impl NetlinkRequestData for NetlinkRouteAttr {
    fn len(&self) -> usize {
        self.rt_attr.rta_len as usize
    }

    fn is_empty(&self) -> bool {
        self.rt_attr.rta_len == 0
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.rt_attr.rta_len.to_ne_bytes());
        buf.extend_from_slice(&self.rt_attr.rta_type.to_ne_bytes());
        buf.extend_from_slice(&self.value);

        let align_to = align_of(buf.len(), consts::RTA_ALIGNTO);
        if buf.len() < align_to {
            buf.resize(align_to, 0);
        }

        if let Some(children) = &self.children {
            for child in children {
                buf.extend_from_slice(&child.serialize()?);
            }
        }

        let len = buf.len();
        buf[..2].copy_from_slice(&(len as u16).to_ne_bytes());

        Ok(buf)
    }
}

impl NetlinkRouteAttr {
    pub fn new(rta_type: u16, value: Vec<u8>) -> Self {
        Self {
            rt_attr: RtAttr {
                rta_len: (consts::RT_ATTR_SIZE + value.len()) as u16,
                rta_type,
            },
            value,
            children: None,
        }
    }

    pub fn map(mut buf: &[u8]) -> Result<HashMap<u16, Vec<u8>>> {
        let mut attrs = HashMap::new();

        while buf.len() >= consts::RT_ATTR_SIZE {
            let rt_attr = unsafe { *(buf.as_ptr() as *const RtAttr) };
            let len = align_of(rt_attr.rta_len as usize, consts::RTA_ALIGNTO);
            let value = buf[consts::RT_ATTR_SIZE..rt_attr.rta_len as usize].to_vec();

            attrs.insert(rt_attr.rta_type, value);
            buf = &buf[len..];
        }

        Ok(attrs)
    }

    pub fn from(mut buf: &[u8]) -> Result<Vec<Self>> {
        let mut attrs = Vec::new();

        while buf.len() >= consts::RT_ATTR_SIZE {
            let rt_attr = unsafe { *(buf.as_ptr() as *const RtAttr) };
            let len = align_of(rt_attr.rta_len as usize, consts::RTA_ALIGNTO);
            let value = buf[consts::RT_ATTR_SIZE..rt_attr.rta_len as usize].to_vec();

            attrs.push(Self {
                rt_attr,
                value,
                children: None,
            });
            buf = &buf[len..];
        }

        Ok(attrs)
    }

    pub fn add_child(&mut self, rta_type: u16, value: Vec<u8>) {
        let attr = Box::new(NetlinkRouteAttr::new(rta_type, value));
        self.rt_attr.rta_len += attr.len() as u16;

        match &mut self.children {
            None => self.children = Some(vec![attr]),
            Some(children) => children.push(attr),
        }
    }

    pub fn add_child_from_attr<T>(&mut self, attr: Box<T>)
    where
        T: NetlinkRequestData + 'static,
    {
        self.rt_attr.rta_len += attr.len() as u16;

        match &mut self.children {
            None => self.children = Some(vec![attr]),
            Some(children) => children.push(attr),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RtAttr {
    pub rta_len: u16,
    pub rta_type: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct InfoMessage {
    pub family: u8,
    pub _pad: u8,
    pub ifi_type: u16,
    pub index: i32,
    pub flags: u32,
    pub change: u32,
}

impl NetlinkRequestData for InfoMessage {
    fn len(&self) -> usize {
        consts::IF_INFO_MSG_SIZE
    }

    fn is_empty(&self) -> bool {
        self.family == 0
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

impl InfoMessage {
    pub fn new(family: i32) -> Self {
        Self {
            family: family as u8,
            ..Default::default()
        }
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        Ok(unsafe { *(buf[..consts::IF_INFO_MSG_SIZE].as_ptr() as *const Self) })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct AddressMessage {
    pub family: u8,
    pub prefix_len: u8,
    pub flags: u8,
    pub scope: u8,
    pub index: i32,
}

impl NetlinkRequestData for AddressMessage {
    fn len(&self) -> usize {
        consts::IF_ADDR_MSG_SIZE
    }

    fn is_empty(&self) -> bool {
        self.family == 0
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

impl AddressMessage {
    pub fn new(family: i32) -> Self {
        Self {
            family: family as u8,
            ..Default::default()
        }
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        Ok(unsafe { *(buf[..consts::IF_ADDR_MSG_SIZE].as_ptr() as *const Self) })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct RouteMessage {
    pub family: u8,
    pub dst_len: u8,
    pub src_len: u8,
    pub tos: u8,
    pub table: u8,
    pub protocol: u8,
    pub scope: u8,
    pub rtm_type: u8,
    pub flags: u32,
}

impl NetlinkRequestData for RouteMessage {
    fn len(&self) -> usize {
        consts::ROUTE_MSG_SIZE
    }

    fn is_empty(&self) -> bool {
        self.family == 0
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

impl RouteMessage {
    pub fn new_rt_msg() -> Self {
        Self {
            table: libc::RT_TABLE_MAIN,
            protocol: libc::RTPROT_BOOT,
            scope: libc::RT_SCOPE_UNIVERSE,
            rtm_type: libc::RTN_UNICAST,
            ..Default::default()
        }
    }

    pub fn new_rt_del_msg() -> Self {
        Self {
            table: libc::RT_TABLE_MAIN,
            scope: libc::RT_SCOPE_NOWHERE,
            ..Default::default()
        }
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        Ok(unsafe { *(buf[..consts::ROUTE_MSG_SIZE].as_ptr() as *const Self) })
    }
}
