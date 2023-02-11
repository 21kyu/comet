use anyhow::{bail, Result};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use netlink_packet_core::{
    NetlinkMessage, NetlinkPayload, NLM_F_ACK, NLM_F_CREATE, NLM_F_DUMP, NLM_F_EXCL, NLM_F_REQUEST,
};
use netlink_packet_route::{
    nlas::link::{Info, InfoKind, Nla},
    AddressMessage, LinkMessage, RouteMessage, RtnlMessage, AF_INET, AF_INET6, IFF_UP, RTN_UNICAST,
    RTPROT_STATIC, RT_SCOPE_UNIVERSE, RT_TABLE_MAIN,
};
use netlink_sys::{protocols::NETLINK_ROUTE, SocketAddr};
use std::net::{Ipv4Addr, Ipv6Addr};

pub struct Socket {
    socket: netlink_sys::Socket,
    sequence_number: u32,
    buffer: [u8; 4096],
}

pub struct LinkOptions {
    name: String,
    kind: InfoKind,
}

pub enum Route {
    V4 { dest: Ipv4Net, gw: Ipv4Addr },
    V6 { dest: Ipv6Net, gw: Ipv6Addr },
}

impl Socket {
    pub fn new() -> Result<Self> {
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

    pub fn add_link(&mut self, options: LinkOptions) -> Result<()> {
        let mut msg = LinkMessage::default();
        let link_info_nlas = vec![Info::Kind(options.kind)];
        msg.nlas.push(Nla::Info(link_info_nlas));
        msg.nlas.push(Nla::IfName(options.name));

        self.request(
            RtnlMessage::NewLink(msg),
            NLM_F_ACK | NLM_F_EXCL | NLM_F_CREATE,
        )?;
        Ok(())
    }

    pub fn set_up(&mut self, name: String) -> Result<()> {
        let mut msg = LinkMessage::default();
        msg.header.flags |= IFF_UP;
        msg.header.change_mask |= IFF_UP;
        msg.nlas.push(Nla::IfName(name));

        self.request(
            RtnlMessage::SetLink(msg),
            NLM_F_ACK | NLM_F_EXCL | NLM_F_CREATE,
        )?;
        Ok(())
    }

    pub fn set_link_name(&mut self, id: u32, name: String) -> Result<()> {
        let mut msg = LinkMessage::default();
        msg.header.index = id;
        msg.nlas.push(Nla::IfName(name));

        self.request(RtnlMessage::SetLink(msg), NLM_F_ACK)?;
        Ok(())
    }

    pub fn add_addr(&mut self, id: u32, addr: &IpNet) -> Result<()> {
        let mut msg = AddressMessage::default();
        msg.header.index = id;
        msg.header.prefix_len = addr.prefix_len();

        let addr_vec = match addr {
            IpNet::V4(v4) => {
                msg.header.family = AF_INET as u8;
                msg.nlas.push(netlink_packet_route::address::Nla::Broadcast(
                    v4.broadcast().octets().to_vec(),
                ));
                v4.addr().octets().to_vec()
            }
            IpNet::V6(v6) => {
                msg.header.family = AF_INET6 as u8;
                v6.addr().octets().to_vec()
            }
        };

        msg.nlas
            .push(netlink_packet_route::address::Nla::Local(addr_vec));

        self.request(
            RtnlMessage::NewAddress(msg),
            NLM_F_ACK | NLM_F_EXCL | NLM_F_CREATE,
        )?;
        Ok(())
    }

    pub fn add_route(&mut self, route: &Route) -> Result<()> {
        let mut msg = RouteMessage::default();

        msg.header.table = RT_TABLE_MAIN;
        msg.header.protocol = RTPROT_STATIC;
        msg.header.scope = RT_SCOPE_UNIVERSE;
        msg.header.kind = RTN_UNICAST;

        let (dest_vec, dest_prefix, gw_vec) = match route {
            Route::V4 { dest, gw } => {
                msg.header.address_family = AF_INET as u8;
                (
                    dest.addr().octets().to_vec(),
                    dest.prefix_len(),
                    gw.octets().to_vec(),
                )
            }
            Route::V6 { dest, gw } => {
                msg.header.address_family = AF_INET6 as u8;
                (
                    dest.addr().octets().to_vec(),
                    dest.prefix_len(),
                    gw.octets().to_vec(),
                )
            }
        };

        msg.header.destination_prefix_length = dest_prefix;
        msg.nlas
            .push(netlink_packet_route::route::Nla::Destination(dest_vec));
        msg.nlas
            .push(netlink_packet_route::route::Nla::Gateway(gw_vec));

        self.request(RtnlMessage::NewRoute(msg), NLM_F_ACK | NLM_F_CREATE)?;
        Ok(())
    }

    fn request(&mut self, msg: RtnlMessage, flags: u16) -> Result<Vec<RtnlMessage>> {
        self.send(msg, flags)?;
        self.recv(flags & NLM_F_DUMP == NLM_F_DUMP)
    }

    fn send(&mut self, msg: RtnlMessage, flags: u16) -> Result<()> {
        let mut packet = NetlinkMessage::from(msg);
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
                    _ => todo!(),
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

impl LinkOptions {
    pub fn new(name: String, kind: InfoKind) -> Self {
        Self { name, kind }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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

    #[test]
    fn test_add_link() {
        test_setup!();
        let mut socket = Socket::new().unwrap();

        let options = LinkOptions::new("test".to_string(), InfoKind::Dummy);
        assert!(socket.add_link(options).is_ok());

        let out = run_command!("ip", "link", "show", "test");
        assert!(out.status.success());
    }

    #[test]
    fn test_set_link_name() {
        test_setup!();
        let mut socket = Socket::new().unwrap();

        let options = LinkOptions::new("test".to_string(), InfoKind::Dummy);
        assert!(socket.add_link(options).is_ok());

        let link = socket.get_link("test".to_string()).unwrap();

        assert!(socket
            .set_link_name(link.header.index, "test2".to_string())
            .is_ok());

        let out = run_command!("ip", "link", "show", "test2");
        assert!(out.status.success());
    }

    #[test]
    fn test_add_addr() {
        test_setup!();
        let mut sock = Socket::new().unwrap();

        let name = "test";
        let opt = LinkOptions::new(name.into(), InfoKind::Dummy);
        assert!(sock.add_link(opt).is_ok());

        let link = sock.get_link(name.into()).unwrap();

        let net = "10.244.0.1/24";
        sock.add_addr(link.header.index, &net.parse().unwrap())
            .unwrap();

        let out = String::from_utf8(run_command!("ip", "addr", "show", "test").stdout).unwrap();
        assert!(out.contains(net));
    }

    #[test]
    fn test_add_route() {
        test_setup!();
        let mut sock = Socket::new().unwrap();

        let name = "test";
        let opt = LinkOptions::new(name.into(), InfoKind::Dummy);
        assert!(sock.add_link(opt).is_ok());
        assert!(sock.set_up(name.into()).is_ok());

        let link = sock.get_link(name.into()).unwrap();
        let net = "10.244.0.3/24";
        assert!(sock
            .add_addr(link.header.index, &net.parse().unwrap())
            .is_ok());

        let gw = "10.244.0.1";

        sock.add_route(&Route::V4 {
            dest: Ipv4Net::from_str("0.0.0.0/0").unwrap(),
            gw: gw.parse().unwrap(),
        })
        .unwrap();

        let out = String::from_utf8(run_command!("ip", "route", "show").stdout).unwrap();
        assert!(out.contains("default via 10.244.0.1 dev test"))
    }
}
