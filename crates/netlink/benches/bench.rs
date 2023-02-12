#![macro_use]
extern crate bencher;

use bencher::{benchmark_group, benchmark_main, Bencher};
use netlink::{
    consts,
    link::link_deserialize,
    message::{IfInfoMessage, NetlinkRouteAttr},
    request::{NetlinkRequest, NetlinkRequestData},
};

fn bench_netlink_route_attr_serialize(b: &mut Bencher) {
    let attr = NetlinkRouteAttr::new(libc::IFLA_IFNAME, "lo".as_bytes().to_vec());
    b.iter(|| {
        attr.serialize().unwrap();
    });
}

fn bench_netlink_request_serialize(b: &mut Bencher) {
    let mut req = NetlinkRequest::new(libc::RTM_GETLINK, libc::NLM_F_ACK);
    let msg = Box::new(IfInfoMessage {
        ifi_family: libc::AF_UNSPEC as u8,
        ifi_index: 1,
        ifi_flags: consts::IFF_UP,
        ifi_change: consts::IFF_UP,
        ..Default::default()
    });

    let name = Box::new(NetlinkRouteAttr::new(
        libc::IFLA_IFNAME,
        "lo".as_bytes().to_vec(),
    ));

    let mtu = Box::new(NetlinkRouteAttr::new(
        libc::IFLA_MTU,
        1500_u32.to_ne_bytes().to_vec(),
    ));

    let mut link_info = Box::new(NetlinkRouteAttr::new(libc::IFLA_LINKINFO, vec![]));
    link_info.add_child(libc::IFLA_INFO_KIND, "dummy".as_bytes().to_vec());

    req.add_data(msg);
    req.add_data(name);
    req.add_data(mtu);
    req.add_data(link_info);

    b.iter(|| {
        let _ = req.serialize().unwrap();
    });
}

fn bench_link_deserialize(b: &mut Bencher) {
    let netlink_msg = [
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

    b.iter(|| {
        let _ = link_deserialize(&netlink_msg).unwrap();
    })
}

benchmark_group!(
    benches,
    bench_netlink_route_attr_serialize,
    bench_netlink_request_serialize,
    bench_link_deserialize
);
benchmark_main!(benches);