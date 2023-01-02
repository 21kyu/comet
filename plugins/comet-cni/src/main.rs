pub mod command;
pub mod connector;
pub mod ipam;
pub mod logging;
pub mod network;

use anyhow::{Error, Result};
use serde::Deserialize;
use std::io::BufRead;
use std::{env, io};

use crate::logging::logging::log;

#[derive(Debug)]
struct Opts {
    command: String,
    netns: String,
    container_id: String,
    if_name: String,
    config: Config,
}

impl Opts {
    fn new<R>(reader: R) -> Result<Self>
    where
        R: BufRead,
    {
        Ok(Self {
            command: env::var("CNI_COMMAND").expect("Failed to get CNI_COMMAND"),
            netns: env::var("CNI_NETNS").expect("Failed to get CNI_NETNS"),
            container_id: env::var("CNI_CONTAINERID").expect("Failed to get CNI_CONTAINERID"),
            if_name: env::var("CNI_IFNAME").expect("Failed to get CNI_IFNAME"),
            config: Config::from(reader)?,
        })
    }

    fn handle(self) -> Result<String> {
        match &self.command[..] {
            "ADD" => Ok(command::add::add(
                &self.if_name,
                &self.container_id,
                &self.config.subnet,
                &self.netns,
            )?),
            "DEL" => command::del::del(&self.if_name, &self.netns),
            "GET" => Ok(String::from("GET not supported")),
            "VERSION" => command::version::version(),
            _ => Err(Error::msg(format!("Unknown CNI command: {}", self.command))),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    name: String,
    network: String,
    subnet: String,
}

impl Config {
    fn from<R>(mut reader: R) -> Result<Self>
    where
        R: BufRead,
    {
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;
        let stdin_json: Self = serde_json::from_str(buffer.as_str())?;
        Ok(stdin_json)
    }
}

fn main() {
    let opts = Opts::new(io::stdin().lock()).unwrap();

    log(&format!("CNI command: {}\n", opts.command));
    log(&format!("stdin: {:?}\n", opts));

    println!("{}", opts.handle().unwrap());
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::Opts;

    #[test]
    fn opts_test() {
        env::set_var("CNI_COMMAND", "ADD");
        env::set_var("CNI_NETNS", "/var/run/netns/123456789");
        env::set_var("CNI_CONTAINERID", "123456789");
        env::set_var("CNI_IFNAME", "eth0");

        let input = r###"
        {
            "cniVersion": "0.4.0",
            "name": "comet",
            "type": "comet-cni",
            "network": "10.244.0.0/16",
            "subnet": "10.244.0.0/24"
        }
        "###
        .as_bytes();

        let opts = Opts::new(&input[..]).unwrap();

        assert_eq!(opts.command, "ADD");
        assert_eq!(opts.netns, "/var/run/netns/123456789");
        assert_eq!(opts.container_id, "123456789");
        assert_eq!(opts.if_name, "eth0");
        assert_eq!(opts.config.name, "comet");
        assert_eq!(opts.config.network, "10.244.0.0/16");
        assert_eq!(opts.config.subnet, "10.244.0.0/24");
    }
}
