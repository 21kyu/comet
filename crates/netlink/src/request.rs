use anyhow::Result;

use crate::socket::NetlinkMessageHeader;

pub(crate) trait NetlinkRequestData {
    fn len(&self) -> usize;
    fn serialize(&self) -> Result<Vec<u8>>;
}

pub(crate) struct NetlinkRequest {
    header: NetlinkMessageHeader,
    data: Option<Vec<Box<dyn NetlinkRequestData>>>,
    raw_data: Option<Vec<u8>>,
}

impl NetlinkRequest {
    pub(crate) fn new(proto: u16, flags: i32) -> Self {
        Self {
            header: NetlinkMessageHeader::new(proto, flags),
            data: None,
            raw_data: None,
        }
    }

    pub(crate) fn serialize(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(bincode::serialize(&self.header)?.as_slice());
        if let Some(data) = &self.data {
            data.iter().for_each(|d| {
                buf.extend_from_slice(&d.serialize().unwrap());
            });
        }
        if let Some(data) = &self.raw_data {
            buf.extend_from_slice(data);
        }
        self.header.nlmsg_len = buf.len() as u32;
        buf[0..4].copy_from_slice(&bincode::serialize(&self.header.nlmsg_len)?.as_slice());

        Ok(buf)
    }

    pub(crate) fn add_data(&mut self, data: Box<dyn NetlinkRequestData>) {
        if self.data.is_none() {
            self.data = Some(Vec::new());
        }
        self.data.as_mut().unwrap().push(data);
    }

    pub(crate) fn add_raw_data(&mut self, data: Vec<u8>) {
        self.raw_data = Some(data);
    }
}

#[cfg(test)]
mod tests {
    use crate::socket::IfInfoMessage;

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
        let msg = IfInfoMessage::deserialize(&NETLINK_MSG).unwrap();
        req.add_data(Box::new(msg));

        let buf = req.serialize().unwrap();
        assert_eq!(buf.len(), 16 + msg.len());
        assert_eq!(req.header.nlmsg_len, 16 + msg.len() as u32);
    }
}
