use std::net::IpAddr;

use ipnet::IpNet;

pub enum RtCmd {
    Add,
    Append,
    Replace,
    Del,
}

#[derive(Default, Debug)]
pub struct Route {
    pub index: i32,
    pub dst: Option<IpNet>,
    pub src: Option<IpAddr>,
    pub gw: Option<IpAddr>,
    pub tos: u8,
    pub table: u8,
    pub protocol: u8,
    pub scope: u8,
    pub rtm_type: u8,
    pub flags: u32,
}
