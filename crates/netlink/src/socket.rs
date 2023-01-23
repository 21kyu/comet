use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::{self, Formatter},
    io::{Error, Result},
    os::fd::RawFd,
};

use crate::{consts, request::NetlinkRequestData, SockAddrNetlink};

pub(crate) struct NetlinkSocket {
    fd: RawFd,
    lsa: SockAddrNetlink,
}

impl NetlinkSocket {
    pub(crate) fn new(protocol: i32, pid: u32, groups: u32) -> Result<Self> {
        let fd = unsafe {
            libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                protocol,
            )
        };
        if fd < 0 {
            return Err(Error::last_os_error());
        }
        let lsa = SockAddrNetlink::new(pid, groups);
        let s = Self { fd, lsa };
        s.bind()?;
        Ok(s)
    }

    fn bind(&self) -> Result<()> {
        let (addr, addr_len) = self.lsa.as_raw();
        let ret = unsafe { libc::bind(self.fd, addr, addr_len) };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    pub(crate) fn send(&self, buf: &[u8]) -> Result<()> {
        let (addr, addr_len) = self.lsa.as_raw();
        let buf_ptr = buf.as_ptr() as *const libc::c_void;
        let buf_len = buf.len() as libc::size_t;
        let ret = unsafe { libc::sendto(self.fd, buf_ptr, buf_len, 0, addr, addr_len) };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    pub(crate) fn recv(&self) -> Result<(Vec<NetlinkMessage>, libc::sockaddr_nl)> {
        let mut from: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        let mut buf: [u8; consts::RECV_BUF_SIZE] = [0; consts::RECV_BUF_SIZE];
        let ret = unsafe {
            libc::recvfrom(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len() as libc::size_t,
                0,
                &mut from as *mut _ as *mut libc::sockaddr,
                &mut std::mem::size_of::<libc::sockaddr_nl>() as *mut _ as *mut libc::socklen_t,
            )
        };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        let netlink_msgs = NetlinkMessage::from(&buf[..ret as usize])?;
        Ok((netlink_msgs, from))
    }

    pub(crate) fn pid(&self) -> Result<u32> {
        let mut rsa: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        let ret = unsafe {
            libc::getsockname(
                self.fd,
                &mut rsa as *mut _ as *mut libc::sockaddr,
                &mut std::mem::size_of::<libc::sockaddr_nl>() as *mut _ as *mut libc::socklen_t,
            )
        };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(rsa.nl_pid)
    }
}

impl Drop for NetlinkSocket {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

pub(crate) struct NetlinkMessage {
    pub(crate) header: NetlinkMessageHeader,
    pub(crate) data: Vec<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, Serialize)]
pub(crate) struct NetlinkMessageHeader {
    pub(crate) nlmsg_len: u32,
    pub(crate) nlmsg_type: u16,
    nlmsg_flags: u16,
    nlmsg_seq: u32,
    pub(crate) nlmsg_pid: u32,
}

impl NetlinkMessageHeader {
    pub(crate) fn new(proto: u16, flags: i32) -> Self {
        Self {
            nlmsg_len: std::mem::size_of::<Self>() as u32,
            nlmsg_type: proto,
            nlmsg_flags: (libc::NLM_F_REQUEST | flags) as u16,
            nlmsg_seq: 1, // TODO
            nlmsg_pid: 0,
        }
    }
}

impl NetlinkMessage {
    fn from(mut buf: &[u8]) -> Result<Vec<Self>> {
        let mut msgs = Vec::new();

        while buf.len() >= consts::NLMSG_HDRLEN {
            let header = unsafe { *(buf.as_ptr() as *const NetlinkMessageHeader) };
            let len = Self::nlm_align_of(header.nlmsg_len as usize);
            let data = buf[consts::NLMSG_HDRLEN..header.nlmsg_len as usize].to_vec();

            msgs.push(Self { header, data });
            buf = &buf[len..];
        }

        Ok(msgs)
    }

    fn nlm_align_of(len: usize) -> usize {
        (len + consts::NLMSG_ALIGNTO - 1) & !(consts::NLMSG_ALIGNTO - 1)
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct IfInfoMessage {
    ifi_family: u8,
    _ifi_pad: u8,
    ifi_type: u16,
    pub(crate) ifi_index: i32,
    pub(crate) ifi_flags: u32,
    ifi_change: u32,
}

impl NetlinkRequestData for IfInfoMessage {
    fn len(&self) -> usize {
        consts::IF_INFO_MSG_SIZE
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

impl IfInfoMessage {
    pub(crate) fn new(family: i32) -> Self {
        Self {
            ifi_family: family as u8,
            _ifi_pad: 0,
            ifi_type: 0,
            ifi_index: 0,
            ifi_flags: 0,
            ifi_change: 0,
        }
    }

    pub(crate) fn deserialize(buf: &[u8]) -> Result<Self> {
        Ok(unsafe { *(buf[..consts::IF_INFO_MSG_SIZE].as_ptr() as *const Self) })
    }
}

#[derive(Serialize)]
pub(crate) struct NetlinkRouteAttr {
    pub(crate) rt_attr: RtAttr,
    #[serde(with = "serde_bytes")]
    pub(crate) value: Vec<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, Serialize)]
pub(crate) struct RtAttr {
    rta_len: u16,
    pub(crate) rta_type: u16,
}

impl NetlinkRequestData for NetlinkRouteAttr {
    fn len(&self) -> usize {
        self.rt_attr.rta_len as usize
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }
}

impl NetlinkRouteAttr {
    pub(crate) fn new(rta_type: u16, value: Vec<u8>) -> Self {
        Self {
            rt_attr: RtAttr {
                rta_len: (consts::RT_ATTR_SIZE + value.len()) as u16,
                rta_type,
            },
            value,
        }
    }

    pub fn map(mut buf: &[u8]) -> Result<HashMap<u16, Vec<u8>>> {
        let mut attrs = HashMap::new();

        while buf.len() >= consts::RT_ATTR_SIZE {
            let rt_attr = unsafe { *(buf.as_ptr() as *const RtAttr) };
            let len = Self::rta_align_of(rt_attr.rta_len as usize);
            let value = buf[consts::RT_ATTR_SIZE..rt_attr.rta_len as usize].to_vec();

            attrs.insert(rt_attr.rta_type, value);
            buf = &buf[len..];
        }

        Ok(attrs)
    }

    pub(crate) fn from(mut buf: &[u8]) -> Result<Vec<Self>> {
        let mut attrs = Vec::new();

        while buf.len() >= consts::RT_ATTR_SIZE {
            let rt_attr = unsafe { *(buf.as_ptr() as *const RtAttr) };
            let len = Self::rta_align_of(rt_attr.rta_len as usize);
            let value = buf[consts::RT_ATTR_SIZE..rt_attr.rta_len as usize].to_vec();

            attrs.push(Self { rt_attr, value });
            buf = &buf[len..];
        }

        Ok(attrs)
    }

    fn parse(m: NetlinkMessage) -> Result<Vec<Self>> {
        let mut b = Vec::new();

        match m.header.nlmsg_type {
            libc::RTM_NEWLINK | libc::RTM_DELLINK => {
                b = m.data[consts::IF_INFO_MSG_SIZE..].to_vec();
            }
            _ => {}
        }

        let mut attrs = Vec::new();
        // TODO
        Ok(attrs)
    }

    fn rta_align_of(len: usize) -> usize {
        (len + consts::RTA_ALIGNTO - 1) & !(consts::RTA_ALIGNTO - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    static NETLINK_MSG: [u8; 96] = [
        0x00, // interface family
        0x00, // reserved
        0x04, 0x03, // link layer type 772 = loopback
        0x01, 0x00, 0x00, 0x00, // interface index = 1
        0x49, 0x00, 0x00, 0x00, // device flags: UP, LOOPBACK, RUNNING, LOWERUP
        0x00, 0x00, 0x00, 0x00, // reserved 2 (aka device change flag)

        // nlas
        0x07, 0x00, 0x03, 0x00, 0x6c, 0x6f, 0x00, // device name L=7,T=3,V=lo
        0x00, // padding
        0x08, 0x00, 0x0d, 0x00, 0xe8, 0x03, 0x00, 0x00, // TxQueue length L=8,T=13,V=1000
        0x05, 0x00, 0x10, 0x00, 0x00, // OperState L=5,T=16,V=0 (unknown)
        0x00, 0x00, 0x00, // padding
        0x05, 0x00, 0x11, 0x00, 0x00, // Link mode L=5,T=17,V=0
        0x00, 0x00, 0x00, // padding
        0x08, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, // MTU L=8,T=4,V=65536
        0x08, 0x00, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, // Group L=8,T=27,V=9
        0x08, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, // Promiscuity L=8,T=30,V=0
        0x08, 0x00, 0x1f, 0x00, 0x01, 0x00, 0x00, 0x00, // Number of Tx Queues L=8,T=31,V=1
        0x08, 0x00, 0x28, 0x00, 0xff, 0xff, 0x00, 0x00, // Maximum GSO segment count L=8,T=40,V=65536
        0x08, 0x00, 0x29, 0x00, 0x00, 0x00, 0x01, 0x00, // Maximum GSO size L=8,T=41,V=65536
    ];

    #[test]
    fn test_if_info_message() {
        let msg = IfInfoMessage::deserialize(&NETLINK_MSG).unwrap();

        assert_eq!(msg.ifi_family, 0);
        assert_eq!(msg._ifi_pad, 0);
        assert_eq!(msg.ifi_type, 772);
        assert_eq!(msg.ifi_index, 1);
        assert_eq!(
            msg.ifi_flags,
            consts::IFF_UP | consts::IFF_LOOPBACK | consts::IFF_RUNNING
        );
        assert_eq!(msg.ifi_change, 0);
    }

    #[test]
    fn test_netlink_socket() {
        let s = NetlinkSocket::new(libc::NETLINK_ROUTE, 0, 0).unwrap();

        // This is a valid message for listing the network links on the system
        let msg = vec![
            0x14, 0x00, 0x00, 0x00, 0x12, 0x00, 0x01, 0x03, 0xfd, 0xfe, 0x38, 0x5c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        s.send(&msg[..]).unwrap();

        let pid = s.pid().unwrap();
        let mut res: Vec<Vec<u8>> = Vec::new();

        'done: loop {
            let (netlink_msgs, from) = s.recv().unwrap();
            println!(
                "received from {} {} {}",
                from.nl_family, from.nl_pid, from.nl_groups
            );

            if from.nl_pid != consts::PID_KERNEL {
                println!("received message from unknown source");
                continue;
            }

            for m in netlink_msgs {
                println!("received message: {:?}", m);

                if m.header.nlmsg_pid != pid {
                    println!("received message with wrong pid");
                    continue;
                }

                match m.header.nlmsg_type {
                    consts::NLMSG_ERROR => {
                        println!("the kernel responded with an error");
                        return;
                    }
                    consts::NLMSG_DONE => {
                        println!("end of dump");
                        break 'done;
                    }
                    _ => {
                        res.push(m.data);
                    }
                }
            }
        }

        println!("res: {:04X?}", res[3]);
        println!("res.len(): {}", res.len());

        res.iter().for_each(|r| {
            let ifi = IfInfoMessage::deserialize(r).unwrap();
            println!("ifi: {:?}", ifi);
        });
    }
}
