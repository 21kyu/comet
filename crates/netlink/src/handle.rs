use anyhow::{bail, Result};

use crate::{
    consts,
    link::{self, Link},
    request::NetlinkRequest,
    socket::NetlinkSocket,
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

    fn link_modify(&mut self, link: &mut dyn Link, flags: i32) -> Result<()> {
        Ok(())
    }

    fn link_get(&mut self, req: &mut NetlinkRequest) -> Result<Box<dyn Link>> {
        let msgs = self.execute(req, 0)?;

        match msgs.len() {
            0 => bail!("no link found"),
            1 => link::link_deserialize(&msgs[0]),
            _ => bail!("multiple links found"),
        }
    }

    fn execute(&mut self, req: &mut NetlinkRequest, res_type: i32) -> Result<Vec<Vec<u8>>> {
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

                        // TODO: print error message
                        bail!("received error message {:?}", m.data);
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
    use crate::{request::NetlinkRequest, socket::IfInfoMessage};

    #[test]
    fn test_link_get() {
        let mut handle = super::SocketHandle::new(libc::NETLINK_ROUTE).unwrap();
        let mut req = NetlinkRequest::new(libc::RTM_GETLINK, libc::NLM_F_ACK);
        let mut msg = Box::new(IfInfoMessage::new(libc::AF_UNSPEC));
        msg.ifi_index = 1;

        req.add_data(msg);

        let link = handle.link_get(&mut req).unwrap();

        assert_eq!(link.attrs().index, 1);
        assert_eq!(link.attrs().name, "lo");
    }
}
