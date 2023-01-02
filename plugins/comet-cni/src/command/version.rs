use anyhow::Result;
use serde_json::json;

pub fn version() -> Result<String> {
    Ok(json!({
        "cniVersion": "0.4.0",
        "supportedVersions": ["0.1.0", "0.2.0", "0.3.0", "0.3.1", "0.4.0"]
    })
    .to_string())
}
