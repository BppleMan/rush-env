use crate::core::script::Scripts;
use crate::visitor::{Visitor, VisitorContext, VisitorError};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Proxy {
    pub host: String,
    pub http_port: String,
    pub socks_port: String,
    pub scripts: Scripts,
}

impl Visitor for Proxy {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        self.scripts.visit(context)
    }
}
