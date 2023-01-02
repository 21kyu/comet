use anyhow::Result;

use crate::connector::veth::release_veth;

pub fn del(if_name: &str, netns: &str) -> Result<String> {
    release_veth(if_name, netns)?;
    Ok(String::from(""))
}
