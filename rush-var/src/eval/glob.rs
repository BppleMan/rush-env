/// 简单的 glob 匹配实现（*, ?, [..]）
pub fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_recursive(pattern, text)
}

fn glob_match_recursive(pattern: &str, text: &str) -> bool {
    let pat_chars: Vec<char> = pattern.chars().collect();
    let txt_chars: Vec<char> = text.chars().collect();

    glob_match_chars(&pat_chars, &txt_chars, 0, 0)
}

fn glob_match_chars(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
    if pi >= pattern.len() {
        return ti >= text.len();
    }

    match pattern[pi] {
        '*' => {
            // 匹配零个或多个字符
            if glob_match_chars(pattern, text, pi + 1, ti) {
                return true;
            }
            if ti < text.len() {
                glob_match_chars(pattern, text, pi, ti + 1)
            } else {
                false
            }
        }
        '?' => {
            if ti < text.len() {
                glob_match_chars(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
        '[' => {
            // 字符类（简化）
            if ti >= text.len() {
                return false;
            }

            let mut end_bracket = pi + 1;
            while end_bracket < pattern.len() && pattern[end_bracket] != ']' {
                end_bracket += 1;
            }

            if end_bracket >= pattern.len() {
                // 无闭合，按字面匹配
                if pattern[pi] == text[ti] {
                    glob_match_chars(pattern, text, pi + 1, ti + 1)
                } else {
                    false
                }
            } else {
                let class = &pattern[pi + 1..end_bracket];
                let ch = text[ti];

                let matches = if !class.is_empty() && class[0] == '!' {
                    // 取反字符类
                    let class_chars = &class[1..];
                    !class_chars.contains(&ch)
                } else {
                    // 常规字符类
                    class.contains(&ch)
                };

                if matches {
                    glob_match_chars(pattern, text, end_bracket + 1, ti + 1)
                } else {
                    false
                }
            }
        }
        _ => {
            if ti < text.len() && pattern[pi] == text[ti] {
                glob_match_chars(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}
