use crate::ast::{Error, Index, ParamExpr, Target};

/// Find and parse all parameter expansions in a string.
/// Returns a list of (start, end, ParamExpr) ranges within the input.
pub fn find_expansions(input: &str) -> Result<Vec<(usize, usize, ParamExpr)>, Error> {
    let mut expansions = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            let start = i;

            if chars[i + 1] == '{' {
                // Find matching closing brace
                let mut brace_count = 0;
                let mut j = i + 1;

                while j < chars.len() {
                    match chars[j] {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                let end = j + 1;
                                let expansion_str: String = chars[start..end].iter().collect();
                                match super::parse_braced(&expansion_str) {
                                    Ok(expr) => expansions.push((start, end, expr)),
                                    Err(e) => return Err(e), // Return error for invalid expansions
                                }
                                i = end;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }

                if brace_count > 0 {
                    // Unmatched braces, skip
                    i += 1;
                }
            } else if chars[i + 1].is_alphabetic() || chars[i + 1] == '_' || chars[i + 1].is_ascii_digit() {
                // Simple $var expansion (including positional parameters like $1, $2)
                let mut j = i + 1;

                if chars[i + 1].is_ascii_digit() {
                    // Handle positional parameters - read consecutive digits
                    while j < chars.len() && chars[j].is_ascii_digit() {
                        j += 1;
                    }
                } else {
                    // Handle regular variables - alphanumeric and underscore
                    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                        j += 1;
                    }
                }

                let name: String = chars[i + 1..j].iter().collect();
                let target = Target {
                    name,
                    special: None,
                    positional: None,
                };
                let expr = ParamExpr::Ref {
                    target,
                    index: Index::None,
                };
                expansions.push((start, j, expr));
                i = j;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    Ok(expansions)
}
