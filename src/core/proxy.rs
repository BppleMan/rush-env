use crate::core::shell_script::function_script::FunctionScript;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Proxy {
    pub proxy_host: String,
    pub http_proxy_port: String,
    pub socks_proxy_port: String,
    pub http_proxy: String,
    pub https_proxy: String,
    pub all_proxy: String,
    pub function: FunctionScript,
}
