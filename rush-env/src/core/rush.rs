use crate::visitor::{Visit, Visitor, VisitorError};
use language::Languages;
use plugin::Plugins;
use proxy::Proxy;
use script::Scripts;
use serde::{Deserialize, Serialize};
use tool::Tools;

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

impl Visit for Rush {
    fn visit<'a>(&'a self, context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        context.section.say(writer, "🌐 Proxy Section 🌐")?;
        self.proxy.visit(context, writer)?;
        writeln!(writer)?;

        context.section.say(writer, "🚀 Plugins Section 🚀")?;
        self.plugins.visit(context, writer)?;
        writeln!(writer)?;

        context.section.say(writer, "🔖 Functions Section  🔖")?;
        self.functions.visit(context, writer)?;
        writeln!(writer)?;

        context.section.say(writer, "✨ Aliases Section ✨")?;
        self.aliases.visit(context, writer)?;
        writeln!(writer)?;

        context.section.say(writer, "🌱 Environment Variables Section 🌱")?;
        self.envs.visit(context, writer)?;
        writeln!(writer)?;

        context.section.say(writer, "🧑‍💻 Languages Section 🧑‍💻")?;
        self.languages.visit(context, writer)?;
        writeln!(writer)?;

        context.section.say(writer, "🛠️ Tools Section 🛠️")?;
        self.tools.visit(context, writer)?;
        writeln!(writer)?;
        Ok(())
    }
}
