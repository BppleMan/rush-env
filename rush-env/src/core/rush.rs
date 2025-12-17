use crate::core::rush::language::Language;
use crate::core::rush::plugin::Plugin;
use crate::core::rush::script::alias::AliasScript;
use crate::core::rush::script::export::ExportScript;
use crate::core::rush::script::function::FunctionScript;
use crate::core::rush::tool::Tool;
use crate::core::rush_context::RushContext;
use color_eyre::Result;
use proxy::Proxy;
use serde::{Deserialize, Serialize};
use tracing::info;

pub mod platform;
pub mod plugin;
pub mod proxy;
pub mod script;
pub mod language;
pub mod condition;
pub mod path;
pub mod tool;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Rush {
    pub proxy: Proxy,
    #[serde(default)]
    pub plugins: Vec<Plugin>,
    #[serde(default)]
    pub functions: Vec<FunctionScript>,
    #[serde(default)]
    pub aliases: Vec<AliasScript>,
    #[serde(default)]
    pub envs: Vec<ExportScript>,
    #[serde(default)]
    pub languages: Vec<Language>,
    #[serde(default)]
    pub tools: Vec<Tool>,
}

impl Rush {
    pub fn install_plugins(&self, context: &mut RushContext) -> Result<()> {
        for plugin in &self.plugins {
            Self::install_plugin(plugin, context)?;
        }

        Ok(())
    }

    fn install_plugin(plugin: &Plugin, context: &mut RushContext) -> Result<()> {
        info!("安装插件: {}", plugin.name);
        plugin.install.install(context)?;
        Ok(())
    }
}

// impl Visit for Rush {
//     fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
//         Bubble::shell(writer).say("🌐 Proxy Section 🌐")?;
//         self.proxy.visit(context, writer)?;
//         writeln!(writer)?;
//
//         Bubble::shell(writer).say("🚀 Plugins Section 🚀")?;
//         self.plugins.visit(context, writer)?;
//         writeln!(writer)?;
//
//         Bubble::shell(writer).say("🔖 Functions Section  🔖")?;
//         self.functions.visit(context, writer)?;
//         writeln!(writer)?;
//
//         Bubble::shell(writer).say("✨ Aliases Section ✨")?;
//         self.aliases.visit(context, writer)?;
//         writeln!(writer)?;
//
//         Bubble::shell(writer).say("🌱 Environment Variables Section 🌱")?;
//         self.envs.visit(context, writer)?;
//         writeln!(writer)?;
//
//         Bubble::shell(writer).say("🧑‍💻 Languages Section 🧑‍💻")?;
//         self.languages.visit(context, writer)?;
//         writeln!(writer)?;
//
//         Bubble::shell(writer).say("🛠️ Tools Section 🛠️")?;
//         self.tools.visit(context, writer)?;
//         writeln!(writer)?;
//         Ok(())
//     }
// }
