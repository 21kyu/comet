pub const NLMSG_ALIGNTO: usize = 0x4;
pub const RTA_ALIGNTO: usize = 0x4;

pub const NLMSG_ERROR: u16 = 2;
pub const NLMSG_DONE: u16 = 3;
pub const NLMSG_HDRLEN: usize = 0x10;

pub const NLA_F_NESTED: u16 = 0x8000;

pub const RECV_BUF_SIZE: usize = 65536;
pub const PID_KERNEL: u32 = 0;

pub const IFF_UP: u32 = 0x1;
pub const IFF_BROADCAST: u32 = 0x2;
pub const IFF_DEBUG: u32 = 0x4;
pub const IFF_LOOPBACK: u32 = 0x8;
pub const IFF_POINTOPOINT: u32 = 0x10;
pub const IFF_NOTRAILERS: u32 = 0x20;
pub const IFF_RUNNING: u32 = 0x40;

pub const RT_ATTR_SIZE: usize = 0x4;
pub const IF_INFO_MSG_SIZE: usize = 0x10;

pub const IFLA_BR_HELLO_TIME: u16 = 0x2;
pub const IFLA_BR_AGEING_TIME: u16 = 0x4;
pub const IFLA_BR_VLAN_FILTERING: u16 = 0x7;
pub const IFLA_BR_MCAST_SNOOPING: u16 = 0x17;

pub const IFLA_XDP_FD: u16 = 0x1;
pub const IFLA_XDP_ATTACHED: u16 = 0x2;
pub const IFLA_XDP_FLAGS: u16 = 0x3;
pub const IFLA_XDP_PROG_ID: u16 = 0x4;

pub const IFLA_GRO_MAX_SIZE: u16 = 0x3a;

pub const VETH_INFO_PEER: u16 = 1;
