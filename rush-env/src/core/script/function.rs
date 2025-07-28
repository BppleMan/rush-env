use crate::core::condition::Condition;
use crate::visitor::{Visit, Visitor, VisitorError};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FunctionScript {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub body: String,
    #[serde(default)]
    pub condition: Condition,
}

impl FunctionScript {
    pub fn tag() -> &'static str {
        "<function name>"
    }
}

impl Visit for FunctionScript {
    fn visit<'a>(&'a self, _context: &mut Visitor<'a>, writer: &mut impl std::io::Write) -> Result<(), VisitorError> {
        if !self.condition.check() {
            return Ok(());
        }
        writeln!(writer, "function {} {{", self.name)?;
        // let lines = self.body.trim_start_matches('\n').trim_end();
        let body = re_indent(&self.body, "    ");
        writeln!(writer, "{body}")?;
        writeln!(writer, "}}")?;
        Ok(())
    }
}

/// 对多行文本重新缩进：
/// 1. 先去除最小公共前缀缩进
/// 2. 再按给定缩进重新缩进每一行
///
/// # 参数
/// - text: 输入多行字符串
/// - indent: 目标缩进字符串，比如 `"  "` 或 `"\t"`
///
/// # 返回
/// 重新缩进后的字符串
pub fn re_indent(text: &str, indent: &str) -> String {
    let lines: Vec<&str> = text.trim_start_matches('\n').trim_end().lines().collect();
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| *c == ' ' || *c == '\t').count())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                // 空行不缩进
                "".to_owned()
            } else {
                format!("{}{}", indent, &line[min_indent..])
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}
