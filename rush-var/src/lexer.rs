pub fn read_until_unescaped(input: &str, delim: char) -> (String, &str) {
    let mut out = String::new();
    let mut escape = false;
    let mut idx = 0;
    for ch in input.chars() {
        if escape {
            out.push(ch);
            escape = false;
        } else if ch == '\\' {
            escape = true;
        } else if ch == delim {
            return (out, &input[idx + ch.len_utf8()..]);
        } else {
            out.push(ch);
        }
        idx += ch.len_utf8();
    }
    (out, &input[idx..])
}

pub fn parse_int(input: &str) -> (i64, &str) {
    let mut sign = 1;
    let mut chars = input;
    if let Some(rest) = chars.strip_prefix('-') {
        sign = -1;
        chars = rest;
    }
    let mut val: i64 = 0;
    let mut consumed = 0;
    for ch in chars.chars() {
        if let Some(d) = ch.to_digit(10) {
            val = val * 10 + d as i64;
            consumed += ch.len_utf8();
        } else {
            break;
        }
    }
    (sign * val, &chars[consumed..])
}
