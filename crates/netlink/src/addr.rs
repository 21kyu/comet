use std::net::IpAddr;

use ipnet::IpNet;

#[derive(Default)]
pub struct Address {
    pub index: i32,
    pub ip: IpNet,
    pub label: String,
    pub flags: i32,
    pub scope: i32,
    pub broadcast: Option<IpAddr>,
    pub peer: Option<IpNet>,
    pub preferred_lifetime: i32,
    pub valid_lifetime: i32,
}
