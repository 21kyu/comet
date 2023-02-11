use std::collections::HashMap;

use anyhow::Result;

use crate::{
    handle::SocketHandle,
    link::{Link, LinkAttrs},
};

const SUPPORTED_PROTOCOLS: [i32; 1] = [libc::NETLINK_ROUTE];

pub struct Netlink {
    pub sockets: HashMap<i32, SocketHandle>,
}

impl Netlink {
    pub fn new() -> Result<Self> {
        let sockets = SUPPORTED_PROTOCOLS
            .iter()
            .map(|proto| Ok((*proto, SocketHandle::new(*proto)?)))
            .collect::<Result<HashMap<i32, SocketHandle>>>()?;

        Ok(Self { sockets })
    }

    pub fn link_get(&mut self, attr: &LinkAttrs) -> Result<Box<dyn Link>> {
        self.sockets
            .entry(libc::NETLINK_ROUTE)
            .or_insert(SocketHandle::new(libc::NETLINK_ROUTE)?)
            .link_get(attr)
    }

    pub fn link_add<T>(&mut self, link: &T) -> Result<()>
    where
        T: Link + ?Sized,
    {
        self.sockets
            .entry(libc::NETLINK_ROUTE)
            .or_insert(SocketHandle::new(libc::NETLINK_ROUTE)?)
            .link_new(
                link,
                libc::NLM_F_CREATE | libc::NLM_F_EXCL | libc::NLM_F_ACK,
            )
    }

    pub fn link_modify<T>(&mut self, link: &T) -> Result<()>
    where
        T: Link + ?Sized,
    {
        self.sockets
            .entry(libc::NETLINK_ROUTE)
            .or_insert(SocketHandle::new(libc::NETLINK_ROUTE)?)
            .link_new(link, libc::NLM_F_ACK)
    }

    pub fn link_del<T>(&mut self, link: &T) -> Result<()>
    where
        T: Link + ?Sized,
    {
        self.sockets
            .entry(libc::NETLINK_ROUTE)
            .or_insert(SocketHandle::new(libc::NETLINK_ROUTE)?)
            .link_del(link)
    }
}

#[cfg(test)]
mod tests {
    use crate::link::Kind;

    use super::*;

    macro_rules! test_setup {
        () => {
            if !nix::unistd::geteuid().is_root() {
                eprintln!("Test skipped, must be run as root");
                return;
            }
            nix::sched::unshare(nix::sched::CloneFlags::CLONE_NEWNET).unwrap();
        };
    }

    #[test]
    fn test_new() {
        test_setup!();
        let nl = Netlink::new().unwrap();
        assert_eq!(nl.sockets.len(), SUPPORTED_PROTOCOLS.len());
    }

    #[test]
    fn test_link_add_modify_del() {
        test_setup!();
        let mut netlink = Netlink::new().unwrap();

        let dummy = Kind::Dummy(LinkAttrs {
            name: "foo".to_string(),
            ..Default::default()
        });

        netlink.link_add(&dummy).unwrap();

        let mut link = netlink.link_get(dummy.attrs()).unwrap();
        assert_eq!(link.attrs().name, "foo");
        assert_eq!(link.link_type(), "dummy");

        link.attrs_mut().name = "bar".to_string();
        netlink.link_modify(&*link).unwrap();

        let link = netlink.link_get(link.attrs()).unwrap();
        assert_eq!(link.attrs().name, "bar");

        netlink.link_del(&*link).unwrap();

        let link = netlink.link_get(link.attrs()).err();
        assert!(link.is_some());
    }
}
