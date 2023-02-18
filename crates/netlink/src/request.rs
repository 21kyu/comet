use anyhow::Result;

use crate::message::NetlinkMessageHeader;

pub trait NetlinkRequestData {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn serialize(&self) -> Result<Vec<u8>>;
}

pub struct NetlinkRequest {
    pub header: NetlinkMessageHeader,
    pub data: Option<Vec<Box<dyn NetlinkRequestData>>>,
    pub raw_data: Option<Vec<u8>>,
}

impl NetlinkRequest {
    pub fn new(proto: u16, flags: i32) -> Self {
        Self {
            header: NetlinkMessageHeader::new(proto, flags),
            data: None,
            raw_data: None,
        }
    }

    pub fn serialize(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend(bincode::serialize(&self.header)?);

        if let Some(data) = &self.data {
            for d in data {
                buf.extend(d.serialize()?);
            }
        }
        if let Some(data) = &self.raw_data {
            buf.extend(data);
        }

        let len = buf.len() as u16;
        buf[..2].copy_from_slice(&len.to_ne_bytes());

        Ok(buf)
    }

    pub fn add_data(&mut self, data: Box<dyn NetlinkRequestData>) {
        self.header.nlmsg_len += data.len() as u32;
        if self.data.is_none() {
            self.data = Some(vec![data]);
        } else if let Some(d) = &mut self.data {
            d.push(data);
        }
    }

    pub fn add_raw_data(&mut self, data: Vec<u8>) {
        self.header.nlmsg_len += data.len() as u32;
        if self.raw_data.is_none() {
            self.raw_data = Some(data);
        } else if let Some(d) = &mut self.raw_data {
            d.extend(data);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{message::InfoMessage, message::NetlinkRouteAttr};

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
    fn test_netlink_request() {
        let mut req = NetlinkRequest::new(0, 0);
        let msg = InfoMessage::deserialize(&NETLINK_MSG).unwrap();
        req.add_data(Box::new(msg));

        let name = NetlinkRouteAttr::new(libc::IFLA_IFNAME, "lo".as_bytes().to_vec());
        req.add_data(Box::new(name));

        let buf = req.serialize().unwrap();
        assert_eq!(buf.len(), 40);
        assert_eq!(req.header.nlmsg_len, 38);
    }
}
