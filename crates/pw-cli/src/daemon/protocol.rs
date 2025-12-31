use serde::{Deserialize, Serialize};

use crate::types::BrowserKind;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonRequest {
    Ping,
    SpawnBrowser {
        browser: BrowserKind,
        headless: bool,
        port: Option<u16>,
    },
    GetBrowser { port: u16 },
    KillBrowser { port: u16 },
    ListBrowsers,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonResponse {
    Pong,
    Browser { cdp_endpoint: String, port: u16 },
    Browsers { list: Vec<BrowserInfo> },
    Ok,
    Error { code: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    pub port: u16,
    pub browser: BrowserKind,
    pub headless: bool,
    pub created_at: u64,
}
