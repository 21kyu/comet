use netlink::{
    addr::{AddrCmd, Address},
    link::{Kind, Link, LinkAttrs},
    netlink::Netlink,
};

fn main() {
    let mut netlink = Netlink::new().unwrap();

    let dummy = Kind::Dummy(LinkAttrs {
        name: "foo".to_string(),
        ..Default::default()
    });

    netlink.link_add(&dummy).unwrap();

    let link = netlink.link_get(dummy.attrs()).unwrap();

    let mut addr = Address {
        ip: "127.0.0.2/24".parse().unwrap(),
        ..Default::default()
    };

    netlink.addr_handle(AddrCmd::Add, &link, &addr).unwrap();

    let result = netlink.addr_show(&link).unwrap();
    println!("{:?}", result);

    addr.ip = "127.0.0.3/24".parse().unwrap();

    netlink.addr_handle(AddrCmd::Replace, &link, &addr).unwrap();

    let result = netlink.addr_show(&link).unwrap();
    println!("{:?}", result);

    netlink.addr_handle(AddrCmd::Del, &link, &addr).unwrap();

    let result = netlink.addr_show(&link).unwrap();
    println!("{:?}", result);
}
