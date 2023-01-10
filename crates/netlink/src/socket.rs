use libc::c_int;
use std::{
    io::{Error, Result},
    os::fd::RawFd,
};

use crate::SockAddrNetlink;

struct NetlinkSocket {
    fd: RawFd,
    lsa: SockAddrNetlink,
}

impl NetlinkSocket {
    fn new(protocol: c_int, pid: u32, groups: u32) -> Result<Self> {
        let fd = unsafe {
            libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                protocol,
            )
        };

        if fd < 0 {
            return Err(Error::last_os_error());
        }

        let lsa = SockAddrNetlink::new(pid, groups);
        let s = Self { fd, lsa };
        s.bind()?;
        Ok(s)
    }

    fn bind(&self) -> Result<()> {
        let (addr, addr_len) = self.lsa.as_raw();
        let ret = unsafe { libc::bind(self.fd, addr, addr_len) };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    fn send(&self, buf: &[u8]) -> Result<()> {
        let buf_ptr = buf.as_ptr() as *const libc::c_void;
        let buf_len = buf.len() as libc::size_t;
        let ret = unsafe { libc::send(self.fd, buf_ptr, buf_len, 0) };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let buf_ptr = buf.as_mut_ptr() as *mut libc::c_void;
        let buf_len = buf.len() as libc::size_t;
        let ret = unsafe { libc::recv(self.fd, buf_ptr, buf_len, 0) };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(ret as usize)
    }
}

impl Drop for NetlinkSocket {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netlink_socket() {
        let s = NetlinkSocket::new(libc::NETLINK_ROUTE, 0, 0).unwrap();

        // This is a valid message for listing the network links on the system
        let msg = vec![
            0x14, 0x00, 0x00, 0x00, 0x12, 0x00, 0x01, 0x03, 0xfd, 0xfe, 0x38, 0x5c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        s.send(&msg[..]).unwrap();

        let mut buf = vec![0; 4096];

        loop {
            let len = s.recv(&mut &mut buf[..]).unwrap();
            println!("received {:?}", &buf[..len]);

            if buf[4] == 2 && buf[5] == 0 {
                println!("the kernel responded with an error");
                return;
            }

            if buf[4] == 3 && buf[5] == 0 {
                println!("end of dump");
                return;
            }
        }
    }
}
