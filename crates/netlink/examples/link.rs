use anyhow::Result;
use netlink::{
    link::{Kind, Link, LinkAttrs},
    netlink::Netlink,
};

fn main() {
    link_add_modify_del().unwrap();
}

fn link_add_modify_del() -> Result<()> {
    let mut netlink = Netlink::new()?;

    let dummy = Kind::Dummy(LinkAttrs {
        name: "foo".to_string(),
        ..Default::default()
    });

    netlink.link_add(&dummy)?;

    let mut link = netlink.link_get(dummy.attrs())?;
    println!("link name: {}", link.attrs().name);

    link.attrs_mut().name = "bar".to_string();
    netlink.link_modify(&*link)?;

    let link = netlink.link_get(link.attrs())?;
    println!("link name: {}", link.attrs().name);

    netlink.link_del(&*link)?;

    Ok(())
}
