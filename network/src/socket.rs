use anyhow::{bail, Result};
use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_packet_route::{nlas::link::Nla, LinkMessage, RtnlMessage};
use netlink_sys::{protocols::NETLINK_ROUTE, SocketAddr};

struct Socket {
    socket: netlink_sys::Socket,
    sequence_number: u32,
    buffer: [u8; 4096],
}

impl Socket {
    fn new() -> Result<Self> {
        let mut socket = netlink_sys::Socket::new(NETLINK_ROUTE)?;
        let _ = socket.bind_auto()?;
        socket.connect(&SocketAddr::new(0, 0))?;

        Ok(Self {
            socket,
            sequence_number: 0,
            buffer: [0; 4096],
        })
    }

    pub fn get_link(&mut self, name: String) -> Result<LinkMessage> {
        let mut msg = LinkMessage::default();
        msg.nlas.push(Nla::IfName(name));

        let mut result = self.request(RtnlMessage::GetLink(msg), 0)?;
        match result.pop() {
            Some(RtnlMessage::NewLink(m)) => Ok(m),
            _ => bail!("Unexpected response"),
        }
    }

    fn request(&mut self, msg: RtnlMessage, flags: u16) -> Result<Vec<RtnlMessage>> {
        self.send(msg, flags)?;
        self.recv(flags & NLM_F_DUMP == NLM_F_DUMP)
    }

    fn send(&mut self, msg: RtnlMessage, flags: u16) -> Result<()> {
        let mut packet = NetlinkMessage {
            header: NetlinkHeader::default(),
            payload: NetlinkPayload::from(msg),
        };
        packet.header.flags = NLM_F_REQUEST | flags;
        packet.header.sequence_number = {
            self.sequence_number += 1;
            self.sequence_number
        };
        packet.finalize();

        packet.serialize(&mut self.buffer[..]);

        self.socket.send(&self.buffer[..packet.buffer_len()], 0)?;
        Ok(())
    }

    fn recv(&mut self, multi: bool) -> Result<Vec<RtnlMessage>> {
        let mut result = Vec::new();
        let mut offset = 0;

        loop {
            let size = self.socket.recv(&mut &mut self.buffer[..], 0)?;

            loop {
                let bytes = &self.buffer[offset..];
                let rx_packet: NetlinkMessage<RtnlMessage> = NetlinkMessage::deserialize(bytes)?;

                if rx_packet.header.sequence_number != self.sequence_number {
                    bail!("Unexpected sequence number");
                }

                match rx_packet.payload {
                    NetlinkPayload::Done => return Ok(result),
                    NetlinkPayload::Noop => bail!("Netlink error: Noop"),
                    NetlinkPayload::Overrun(_) => bail!("Netlink error: Overrun"),
                    NetlinkPayload::Error(e) | NetlinkPayload::Ack(e) => {
                        if e.code != 0 {
                            bail!("Netlink error: {}", e);
                        }
                        return Ok(result);
                    }
                    NetlinkPayload::InnerMessage(msg) => {
                        result.push(msg);
                        if !multi {
                            return Ok(result);
                        }
                    }
                };

                offset += rx_packet.header.length as usize;
                if offset >= size || rx_packet.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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

    macro_rules! run_command {
        ($command:expr $(, $args:expr)*) => {
            std::process::Command::new($command).args([$($args),*]).output()
                .expect("failed to run command")
        };
    }

    #[test]
    fn test_socket_new() {
        test_setup!();
        assert!(Socket::new().is_ok());
    }

    #[test]
    fn test_get_link() {
        test_setup!();
        let mut socket = Socket::new().unwrap();
        let link = socket.get_link("lo".to_string()).unwrap();
        assert_eq!(link.header.index, 1);
    }
}
