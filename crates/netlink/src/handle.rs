use anyhow::{bail, Result};

use crate::{
    consts,
    link::{self, Link, LinkAttrs},
    request::NetlinkRequest,
    socket::{IfInfoMessage, NetlinkRouteAttr, NetlinkSocket},
};

pub struct SocketHandle {
    seq: u32,
    socket: NetlinkSocket,
}

impl SocketHandle {
    fn new(protocol: i32) -> Result<Self> {
        Ok(Self {
            seq: 0,
            socket: NetlinkSocket::new(protocol, 0, 0)?,
        })
    }

    fn link_new(&mut self, link: &mut dyn Link, flags: i32) -> Result<()> {
        let base = link.attrs();
        let mut req = NetlinkRequest::new(libc::RTM_NEWLINK, flags);
        let mut msg = Box::new(IfInfoMessage::new(libc::AF_UNSPEC));

        if base.index != 0 {
            msg.ifi_index = base.index;
        }

        if base.flags & consts::IFF_UP != 0 {
            msg.ifi_flags = consts::IFF_UP;
            msg.ifi_change = consts::IFF_UP;
        }
        // TODO: add more flags

        req.add_data(msg);

        let name = Box::new(NetlinkRouteAttr::new(
            libc::IFLA_IFNAME,
            base.name.as_bytes().to_vec(),
        ));

        req.add_data(name);

        Ok(())
    }

    fn link_get(&mut self, attr: &LinkAttrs) -> Result<Box<dyn Link>> {
        let mut req = NetlinkRequest::new(libc::RTM_GETLINK, libc::NLM_F_ACK);
        let mut msg = Box::new(IfInfoMessage::new(libc::AF_UNSPEC));

        if attr.index != 0 {
            msg.ifi_index = attr.index;
        }

        req.add_data(msg);

        //let ext_mask = Box::new(NetlinkRouteAttr::new(libc::IFLA_EXT_MASK, vec![0, 0, 0, 1]));
        //req.add_data(ext_mask);

        if attr.name != "" {
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
                        let err_no = i32::from_ne_bytes(m.data[0..4].try_into().unwrap());

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
    use crate::link;

    #[test]
    fn test_link_get() {
        let mut handle = super::SocketHandle::new(libc::NETLINK_ROUTE).unwrap();
        let mut attr = link::LinkAttrs::new();
        attr.name = "lo".to_string();

        let link = handle.link_get(&attr).unwrap();

        assert_eq!(link.attrs().index, 1);
        assert_eq!(link.attrs().name, "lo");
    }
}
