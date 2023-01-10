pub struct SockAddrNetlink(libc::sockaddr_nl);

impl SockAddrNetlink {
    pub fn new(pid: u32, groups: u32) -> Self {
        let mut addr: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        addr.nl_family = libc::AF_NETLINK as u16;
        addr.nl_pid = pid;
        addr.nl_groups = groups;
        Self(addr)
    }

    pub fn as_raw(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        (
            &self.0 as *const _ as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        )
    }
}
