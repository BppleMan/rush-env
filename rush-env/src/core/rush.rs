use crate::core::language::Languages;
use crate::core::plugin::Plugins;
use crate::core::proxy::Proxy;
use crate::core::script::Scripts;
use crate::core::tool::Tools;
use crate::visitor::{Visit, Visitor, VisitorError};
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
