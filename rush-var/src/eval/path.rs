use std::path::Path;
use crate::ast::PathMod;

/// 路径修饰符应用
pub fn apply_path_modifier(value: &str, modifier: &PathMod) -> String {
    let path = Path::new(value);

    match modifier {
        PathMod::H => {
            // dirname
            if value.is_empty() {
                ".".to_string()
            } else {
                path.parent()
                    .map(|p| {
                        let parent_str = p.to_string_lossy();
                        if parent_str.is_empty() {
                            ".".to_string()
                        } else {
                            parent_str.to_string()
                        }
                    })
                    .unwrap_or_else(|| ".".to_string())
            }
        }
        PathMod::T => {
            // basename
            if value.ends_with('/') && value != "/" {
                "".to_string()
            } else {
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| value.to_string())
            }
        }
        PathMod::R => {
            // root (remove extension)
            path.with_extension("").to_string_lossy().to_string()
        }
        PathMod::E => {
            // extension
            path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()
        }
        PathMod::A | PathMod::LowerA => {
            // realpath - 暂时按原样返回（TODO：后续实现）
            value.to_string()
        }
    }
}
