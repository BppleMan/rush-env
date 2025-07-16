use crate::core::language::Languages;
use crate::core::plugin::Plugins;
use crate::core::proxy::Proxy;
use crate::core::script::Scripts;
use crate::core::tool::Tools;
use crate::visitor::{Visitor, VisitorContext, VisitorError};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Rush {
    pub proxy: Proxy,
    #[serde(default)]
    pub plugins: Plugins,
    #[serde(default)]
    pub functions: Scripts,
    #[serde(default)]
    pub aliases: Scripts,
    #[serde(default)]
    pub envs: Scripts,
    #[serde(default)]
    pub languages: Languages,
    #[serde(default)]
    pub tools: Tools,
}

impl Visitor for Rush {
    fn visit<'a>(&'a self, context: &mut VisitorContext<'a>) -> Result<(), VisitorError> {
        self.proxy.visit(context)?;
        self.plugins.visit(context)?;
        self.functions.visit(context)?;
        self.aliases.visit(context)?;
        self.envs.visit(context)?;
        self.languages.visit(context)?;
        self.tools.visit(context)?;
        Ok(())
    }
}
