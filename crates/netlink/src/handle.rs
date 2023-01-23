use anyhow::{bail, Result};

use crate::{consts, request::NetlinkRequest, socket::NetlinkSocket};

pub struct SocketHandle {
    seq: u32,
    socket: NetlinkSocket,
}

impl SocketHandle {
    fn new(protocol: i32) -> Self {
        Self {
            seq: 0,
            socket: NetlinkSocket::new(protocol, 0, 0).unwrap(),
        }
    }

    fn execute(&self, req: &mut NetlinkRequest) -> Result<Vec<Vec<u8>>> {
        let buf = req.serialize()?;

        println!("sending message: {:?}", buf);
        println!("length: {}", buf.len());

        self.socket.send(&buf)?;

        let pid = self.socket.pid()?;
        let mut res: Vec<Vec<u8>> = Vec::new();

        'done: loop {
            let (msgs, from) = self.socket.recv()?;

            println!(
                "received message from {} {} {}",
                from.nl_family, from.nl_pid, from.nl_groups
            );

            if from.nl_pid != consts::PID_KERNEL {
                println!("received message from unknown source: {}", from.nl_pid);
                continue;
            }

            for msg in msgs {
                if msg.header.nlmsg_pid != pid {
                    println!("received message with wrong pid: {}", msg.header.nlmsg_pid);
                    continue;
                }

                match msg.header.nlmsg_type {
                    consts::NLMSG_DONE => {
                        println!("end of dump");
                        break 'done;
                    }
                    consts::NLMSG_ERROR => {
                        let err_no = i32::from_ne_bytes(msg.data[0..4].try_into().unwrap());

                        println!("received error message: {}", err_no);

                        if err_no == 0 {
                            break 'done;
                        }

                        // TODO: print error message
                        bail!("received error message {:?}", msg.data);
                    }
                    _ => {
                        res.push(msg.data);
                    }
                }
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::{link, request::NetlinkRequest, socket::IfInfoMessage};

    #[test]
    fn test_execute() {
        let handle = super::SocketHandle::new(libc::NETLINK_ROUTE);

        let mut req = NetlinkRequest::new(libc::RTM_GETLINK, libc::NLM_F_ACK);

        let mut msg = Box::new(IfInfoMessage::new(libc::AF_UNSPEC));
        msg.ifi_index = 1;

        req.add_data(msg);

        let res = handle.execute(&mut req).unwrap();

        assert_eq!(res.len(), 1);

        let link = link::link_deserialize(&res[0]).unwrap();

        assert_eq!(link.attrs().index, 1);
        assert_eq!(link.attrs().name, "lo");
    }
}
