pub const NLMSG_ALIGNTO: usize = 4;
pub const NLMSG_ERROR: u16 = libc::NLMSG_ERROR as u16;
pub const NLMSG_DONE: u16 = libc::NLMSG_DONE as u16;
pub const NLMSG_HDRLEN: usize = 16;

pub const RECV_BUF_SIZE: usize = 65536;
pub const PID_KERNEL: u32 = 0;

pub const IFF_UP: u32 = 0x1;
pub const IFF_BROADCAST: u32 = 0x2;
pub const IFF_DEBUG: u32 = 0x4;
pub const IFF_LOOPBACK: u32 = 0x8;
pub const IFF_POINTOPOINT: u32 = 0x10;
pub const IFF_NOTRAILERS: u32 = 0x20;
pub const IFF_RUNNING: u32 = 0x40;

pub const IF_INFO_MSG_SIZE: usize = 0x10;
