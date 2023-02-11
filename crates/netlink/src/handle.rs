use anyhow::{bail, Result};

use crate::{
    consts,
    link::{self, Kind, Link, LinkAttrs, Namespace},
    message::{IfInfoMessage, NetlinkRouteAttr},
    request::NetlinkRequest,
    socket::NetlinkSocket,
    utils::zero_terminated,
};

pub struct SocketHandle {
    pub seq: u32,
    pub socket: NetlinkSocket,
}

impl SocketHandle {
    pub fn new(protocol: i32) -> Result<Self> {
        Ok(Self {
            seq: 0,
            socket: NetlinkSocket::new(protocol, 0, 0)?,
        })
    }

    pub fn link_new<T>(&mut self, link: &T, flags: i32) -> Result<()>
    where
        T: Link + ?Sized,
    {
        let base = link.attrs();
        let mut req = NetlinkRequest::new(libc::RTM_NEWLINK, flags);
        let mut msg = Box::new(IfInfoMessage::new(libc::AF_UNSPEC));

        if base.index != 0 {
            msg.ifi_index = base.index;
        }

        // TODO: add more flags
        if base.flags & consts::IFF_UP != 0 {
            msg.ifi_flags = consts::IFF_UP;
            msg.ifi_change = consts::IFF_UP;
        }

        req.add_data(msg);

        let name = Box::new(NetlinkRouteAttr::new(
            libc::IFLA_IFNAME,
            zero_terminated(&base.name),
        ));

        req.add_data(name);

        // TODO
        // if base.hw_addr.len() > 0 {
        //     let hw_addr = Box::new(NetlinkRouteAttr::new(
        //         libc::IFLA_ADDRESS,
        //         base.hw_addr.clone(),
        //     ));
        //     req.add_data(hw_addr);
        // }

        if base.mtu > 0 {
            let mtu = Box::new(NetlinkRouteAttr::new(
                libc::IFLA_MTU,
                base.mtu.to_ne_bytes().to_vec(),
            ));
            req.add_data(mtu);
        }

        if base.tx_queue_len > 0 {
            let tx_queue_len = Box::new(NetlinkRouteAttr::new(
                libc::IFLA_TXQLEN,
                base.tx_queue_len.to_ne_bytes().to_vec(),
            ));
            req.add_data(tx_queue_len);
        }

        if base.num_tx_queues > 0 {
            let num_tx_queues = Box::new(NetlinkRouteAttr::new(
                libc::IFLA_NUM_TX_QUEUES,
                base.num_tx_queues.to_ne_bytes().to_vec(),
            ));
            req.add_data(num_tx_queues);
        }

        if base.num_rx_queues > 0 {
            let num_rx_queues = Box::new(NetlinkRouteAttr::new(
                libc::IFLA_NUM_RX_QUEUES,
                base.num_rx_queues.to_ne_bytes().to_vec(),
            ));
            req.add_data(num_rx_queues);
        }

        let mut link_info = Box::new(NetlinkRouteAttr::new(libc::IFLA_LINKINFO, vec![]));

        link_info.add_child(libc::IFLA_INFO_KIND, link.link_type().as_bytes().to_vec());

        match link.kind() {
            Kind::Bridge {
                attrs: _,
                hello_time,
                ageing_time,
                multicast_snooping,
                vlan_filtering,
            } => {
                let mut data = Box::new(NetlinkRouteAttr::new(libc::IFLA_INFO_DATA, vec![]));

                if let Some(hello_time) = hello_time {
                    data.add_child(
                        consts::IFLA_BR_HELLO_TIME,
                        hello_time.to_ne_bytes().to_vec(),
                    );
                }

                if let Some(ageing_time) = ageing_time {
                    data.add_child(
                        consts::IFLA_BR_AGEING_TIME,
                        ageing_time.to_ne_bytes().to_vec(),
                    );
                }

                if let Some(multicast_snooping) = multicast_snooping {
                    data.add_child(
                        consts::IFLA_BR_MCAST_SNOOPING,
                        (*multicast_snooping as u8).to_ne_bytes().to_vec(),
                    );
                }

                if let Some(vlan_filtering) = vlan_filtering {
                    data.add_child(
                        consts::IFLA_BR_VLAN_FILTERING,
                        (*vlan_filtering as u8).to_ne_bytes().to_vec(),
                    );
                }

                link_info.add_child_from_attr(data);
            }
            Kind::Veth {
                attrs: _,
                peer_name,
                peer_hw_addr,
                peer_ns,
            } => {
                let mut data = Box::new(NetlinkRouteAttr::new(libc::IFLA_INFO_DATA, vec![]));
                let mut peer_info = Box::new(NetlinkRouteAttr::new(consts::VETH_INFO_PEER, vec![]));

                peer_info.add_child_from_attr(Box::new(IfInfoMessage::new(libc::AF_UNSPEC)));
                peer_info.add_child(libc::IFLA_IFNAME, zero_terminated(peer_name));

                if base.mtu > 0 {
                    peer_info.add_child(libc::IFLA_MTU, base.mtu.to_ne_bytes().to_vec());
                }

                if base.tx_queue_len >= 0 {
                    peer_info
                        .add_child(libc::IFLA_TXQLEN, base.tx_queue_len.to_ne_bytes().to_vec());
                }

                if base.num_tx_queues > 0 {
                    peer_info.add_child(
                        libc::IFLA_NUM_TX_QUEUES,
                        base.num_tx_queues.to_ne_bytes().to_vec(),
                    );
                }

                if base.num_rx_queues > 0 {
                    peer_info.add_child(
                        libc::IFLA_NUM_RX_QUEUES,
                        base.num_rx_queues.to_ne_bytes().to_vec(),
                    );
                }

                if let Some(hw_addr) = peer_hw_addr {
                    peer_info.add_child(libc::IFLA_ADDRESS, hw_addr.to_vec());
                }

                match peer_ns {
                    Some(ns) => match ns {
                        Namespace::Pid(pid) => {
                            peer_info.add_child(libc::IFLA_NET_NS_PID, pid.to_ne_bytes().to_vec());
                        }
                        Namespace::Fd(fd) => {
                            peer_info.add_child(libc::IFLA_NET_NS_FD, fd.to_ne_bytes().to_vec());
                        }
                    },
                    None => {}
                }

                data.add_child_from_attr(peer_info);
                link_info.add_child_from_attr(data);
            }
            _ => {}
        }

        req.add_data(link_info);

        let _ = self.execute(&mut req, 0)?;

        Ok(())
    }

    pub fn link_del<T>(&mut self, link: &T) -> Result<()>
    where
        T: Link + ?Sized,
    {
        let base = link.attrs();

        let mut req = NetlinkRequest::new(libc::RTM_DELLINK, libc::NLM_F_ACK);

        let mut msg = Box::new(IfInfoMessage::new(libc::AF_UNSPEC));
        msg.ifi_index = base.index;

        req.add_data(msg);

        let _ = self.execute(&mut req, 0)?;

        Ok(())
    }

    pub fn link_get(&mut self, attr: &LinkAttrs) -> Result<Box<dyn Link>> {
        let mut req = NetlinkRequest::new(libc::RTM_GETLINK, libc::NLM_F_ACK);
        let mut msg = Box::new(IfInfoMessage::new(libc::AF_UNSPEC));

        if attr.index != 0 {
            msg.ifi_index = attr.index;
        }

        req.add_data(msg);

        if !attr.name.is_empty() {
            let name = Box::new(NetlinkRouteAttr::new(
                libc::IFLA_IFNAME,
                attr.name.as_bytes().to_vec(),
            ));
            req.add_data(name);
        }

        let msgs = self.execute(&mut req, 0)?;

        match msgs.len() {
            0 => bail!("no link found"),
            1 => link::link_deserialize(&msgs[0]),
            _ => bail!("multiple links found"),
        }
    }

    fn execute(&mut self, req: &mut NetlinkRequest, _res_type: i32) -> Result<Vec<Vec<u8>>> {
        req.header.nlmsg_seq = {
            self.seq += 1;
            self.seq
        };

        let buf = req.serialize()?;

        self.socket.send(&buf)?;

        let pid = self.socket.pid()?;
        let mut res: Vec<Vec<u8>> = Vec::new();

        'done: loop {
            let (msgs, from) = self.socket.recv()?;

            if from.nl_pid != consts::PID_KERNEL {
                bail!(
                    "wrong sender pid: {}, expected: {}",
                    from.nl_pid,
                    consts::PID_KERNEL
                );
            }

            for m in msgs {
                if m.header.nlmsg_seq != req.header.nlmsg_seq {
                    bail!(
                        "wrong sequence number: {}, expected: {}",
                        m.header.nlmsg_seq,
                        req.header.nlmsg_seq
                    );
                }

                if m.header.nlmsg_pid != pid {
                    continue;
                }

                match m.header.nlmsg_type {
                    consts::NLMSG_DONE | consts::NLMSG_ERROR => {
                        let err_no = i32::from_ne_bytes(m.data[0..4].try_into()?);

                        if err_no == 0 {
                            break 'done;
                        }

                        let err_msg = unsafe { std::ffi::CStr::from_ptr(libc::strerror(-err_no)) };
                        bail!("{} ({}): {:?}", err_msg.to_str()?, -err_no, &m.data[4..]);
                    }
                    _ => {
                        res.push(m.data);
                    }
                }
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::link::{self, Kind, LinkAttrs};

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
    fn test_link_add_modify_del() {
        test_setup!();
        let mut handle = super::SocketHandle::new(libc::NETLINK_ROUTE).unwrap();
        let mut attr = LinkAttrs::new();
        attr.name = "foo".to_string();

        let link = Kind::Dummy(attr.clone());

        handle
            .link_new(
                &link,
                libc::NLM_F_CREATE | libc::NLM_F_EXCL | libc::NLM_F_ACK,
            )
            .unwrap();

        let link = handle.link_get(&attr).unwrap();
        assert_eq!(link.attrs().name, "foo");

        attr = link.attrs().clone();
        attr.name = "bar".to_string();

        let link = Kind::Dummy(attr.clone());

        handle.link_new(&link, libc::NLM_F_ACK).unwrap();

        let link = handle.link_get(&attr).unwrap();
        assert_eq!(link.attrs().name, "bar");

        handle.link_del(&*link).unwrap();

        let res = handle.link_get(&attr).err();
        println!("{res:?}");
        assert!(res.is_some());
    }

    #[test]
    fn test_link_bridge() {
        test_setup!();
        let mut handle = super::SocketHandle::new(libc::NETLINK_ROUTE).unwrap();
        let mut attr = LinkAttrs::new();
        attr.name = "foo".to_string();

        let link = Kind::Bridge {
            attrs: attr.clone(),
            hello_time: None,
            ageing_time: Some(30102),
            multicast_snooping: None,
            vlan_filtering: Some(true),
        };

        handle
            .link_new(
                &link,
                libc::NLM_F_CREATE | libc::NLM_F_EXCL | libc::NLM_F_ACK,
            )
            .unwrap();

        let link = handle.link_get(&attr).unwrap();
        assert_eq!(link.attrs().link_type, "bridge");
        assert_eq!(link.attrs().name, "foo");

        match link.kind() {
            Kind::Bridge {
                attrs: _,
                hello_time,
                ageing_time,
                multicast_snooping,
                vlan_filtering,
            } => {
                assert_eq!(hello_time.unwrap(), 200);
                assert_eq!(ageing_time.unwrap(), 30102);
                assert!(multicast_snooping.unwrap());
                assert!(vlan_filtering.unwrap());
            }
            _ => panic!("wrong link type"),
        }

        handle.link_del(&*link).unwrap();

        let res = handle.link_get(&attr).err();
        assert!(res.is_some());
    }

    #[test]
    fn test_link_veth() {
        test_setup!();
        let mut handle = super::SocketHandle::new(libc::NETLINK_ROUTE).unwrap();
        let mut attr = LinkAttrs::new();
        attr.name = "foo".to_string();
        attr.mtu = 1400;
        attr.tx_queue_len = 100;
        attr.num_tx_queues = 4;
        attr.num_rx_queues = 8;

        // TODO: need to set peer hw addr and peer ns
        let link = Kind::Veth {
            attrs: attr.clone(),
            peer_name: "bar".to_string(),
            peer_hw_addr: None,
            peer_ns: None,
        };

        handle
            .link_new(
                &link,
                libc::NLM_F_CREATE | libc::NLM_F_EXCL | libc::NLM_F_ACK,
            )
            .unwrap();

        let link = handle.link_get(&attr).unwrap();

        let peer = handle
            .link_get(&LinkAttrs {
                name: "bar".to_string(),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(link.attrs().link_type, "veth");
        assert_eq!(link.attrs().name, "foo");
        assert_eq!(link.attrs().mtu, 1400);
        assert_eq!(link.attrs().tx_queue_len, 100);
        assert_eq!(link.attrs().num_tx_queues, 4);
        assert_eq!(link.attrs().num_rx_queues, 8);

        assert_eq!(peer.attrs().link_type, "veth");
        assert_eq!(peer.attrs().name, "bar");
        assert_eq!(peer.attrs().mtu, 1400);
        assert_eq!(peer.attrs().tx_queue_len, 100);
        assert_eq!(peer.attrs().num_tx_queues, 4);
        assert_eq!(peer.attrs().num_rx_queues, 8);

        handle.link_del(&*peer).unwrap();

        let res = handle.link_get(&attr).err();
        assert!(res.is_some());
    }

    #[test]
    fn test_link_get() {
        test_setup!();
        let mut handle = super::SocketHandle::new(libc::NETLINK_ROUTE).unwrap();
        let mut attr = link::LinkAttrs::new();
        attr.name = "lo".to_string();

        let link = handle.link_get(&attr).unwrap();

        assert_eq!(link.attrs().index, 1);
        assert_eq!(link.attrs().name, "lo");
    }
}
