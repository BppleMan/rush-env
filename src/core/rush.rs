use crate::core::dev_tool::DevTool;
use crate::core::proxy::Proxy;
use crate::core::shell_script::ShellScript;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Rush {
    pub proxy: Proxy,
    #[serde(deserialize_with = "shell_script_list")]
    pub starship: Vec<ShellScript>,
    #[serde(deserialize_with = "shell_script_list")]
    pub antidote: Vec<ShellScript>,
    #[serde(deserialize_with = "shell_script_list")]
    pub functions: Vec<ShellScript>,
    #[serde(deserialize_with = "shell_script_list")]
    pub aliases: Vec<ShellScript>,
    #[serde(deserialize_with = "shell_script_list")]
    pub envs: Vec<ShellScript>,
    #[serde(deserialize_with = "shell_script_list")]
    pub dev_libs: Vec<ShellScript>,
    #[serde(deserialize_with = "dev_tool_list")]
    pub dev_tools: Vec<DevTool>,
}

pub fn shell_script_list<'de, D>(deserializer: D) -> Result<Vec<ShellScript>, D::Error>
where
    D: Deserializer<'de>,
{
    /// Represents <list>...</list>
    #[derive(Deserialize)]
    struct List {
        // default allows empty list
        #[serde(rename = "$value", default)]
        element: Vec<ShellScript>,
    }
    Ok(List::deserialize(deserializer)?.element)
}

pub fn dev_tool_list<'de, D>(deserializer: D) -> Result<Vec<DevTool>, D::Error>
where
    D: Deserializer<'de>,
{
    /// Represents <list>...</list>
    #[derive(Deserialize)]
    struct List {
        // default allows empty list
        #[serde(rename = "$value", default)]
        element: Vec<DevTool>,
    }
    Ok(List::deserialize(deserializer)?.element)
}
