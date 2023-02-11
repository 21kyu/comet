use std::{
    collections::HashMap,
    fmt::{self, Formatter},
};

use anyhow::Result;
use serde::Serialize;

use crate::{consts, request::NetlinkRequestData, utils::align_of};

pub struct NetlinkMessage {
    pub header: NetlinkMessageHeader,
    pub data: Vec<u8>,
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

impl fmt::Debug for NetlinkMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NetlinkMessage {{ header: {} {} {} {} {}, data: {:?} }}",
            self.header.nlmsg_len,
            self.header.nlmsg_type,
            self.header.nlmsg_flags,
            self.header.nlmsg_seq,
            self.header.nlmsg_pid,
            self.data,
        )
    }
}

pub struct NetlinkRouteAttr {
    pub rt_attr: RtAttr,
    pub value: Vec<u8>,
    pub children: Option<Vec<Box<dyn NetlinkRequestData>>>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RtAttr {
    pub rta_len: u16,
    pub rta_type: u16,
}

impl NetlinkRequestData for NetlinkRouteAttr {
    fn len(&self) -> usize {
        self.rt_attr.rta_len as usize
    }

    fn is_empty(&self) -> bool {
        self.rt_attr.rta_len == 0
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
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
#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct IfInfoMessage {
    pub ifi_family: u8,
    _ifi_pad: u8,
    pub ifi_type: u16,
    pub ifi_index: i32,
    pub ifi_flags: u32,
    pub ifi_change: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Serialize)]
pub struct IfAddrMessage {
    pub ifa_family: u8,
    pub ifa_prefix_len: u8,
    pub ifa_flags: u8,
    pub ifa_scope: u8,
    pub ifa_index: u32,
}

impl NetlinkRequestData for IfInfoMessage {
    fn len(&self) -> usize {
        consts::IF_INFO_MSG_SIZE
    }

    fn is_empty(&self) -> bool {
        self.ifi_family == 0
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

impl NetlinkRequestData for IfAddrMessage {
    fn len(&self) -> usize {
        consts::IF_ADDR_MSG_SIZE
    }

    fn is_empty(&self) -> bool {
        self.ifa_family == 0
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

impl IfInfoMessage {
    pub fn new(family: i32) -> Self {
        Self {
            ifi_family: family as u8,
            ..Default::default()
        }
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        Ok(unsafe { *(buf[..consts::IF_INFO_MSG_SIZE].as_ptr() as *const Self) })
    }
}

impl IfAddrMessage {
    pub fn new(family: i32) -> Self {
        Self {
            ifa_family: family as u8,
            ..Default::default()
        }
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        Ok(unsafe { *(buf[..consts::IF_ADDR_MSG_SIZE].as_ptr() as *const Self) })
    }
}
