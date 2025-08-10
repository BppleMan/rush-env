use super::util::{Cursor, is_ident_head, is_ident_tail, push_text_if_any};
use crate::{Span, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Outer,
    Expansion,
    Flags,
    Sep,
    Pattern,
    Subscript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InBracePhase {
    ExpectHead, // 还没读到 head：Ident 或 SpecialParam（此时的 '#' 代表长度）
    AfterHead,  // 已读到 head：此时出现 #/##/%/%% 代表修剪；'/' 可能进入替换
}

fn bump_and_push(cur: &mut Cursor, out: &mut Vec<Token>, ctor: fn(Span) -> Token, offset: usize) {
    let s = cur.i;
    cur.bump();
    out.push(ctor(Span::new(s, s + offset)));
}

/// OUTER：外层文本；遇到 `$` → 进入 Expansion
pub fn scan_outer(cur: &mut Cursor, out: &mut Vec<Token>) -> Mode {
    let mut text_start: Option<usize> = None;

    while let Some(b) = cur.peek() {
        match b {
            b'$' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::dollar, 1);
                return Mode::Expansion;
            }
            _ => {
                if text_start.is_none() {
                    text_start = Some(cur.i);
                } else {
                    cur.bump();
                }
            }
        }
    }

    push_text_if_any(&mut text_start, cur, out);
    Mode::Outer
}

/// EXPANSION：处理 `${...}`、`$ident`、`$special`
pub fn scan_expansion(cur: &mut Cursor, out: &mut Vec<Token>) -> Mode {
    // ${...}
    if cur.peek() == Some(b'{') {
        return scan_inside_brace(cur, out);
    }

    // $special eg. $*, $@, $#, $?, $-, $!, $_, $0..$9
    if let Some(b'*' | b'@' | b'#' | b'?' | b'-' | b'$' | b'!' | b'_' | b'0'..=b'9') = cur.peek() {
        bump_and_push(cur, out, Token::special_param, 1);
        return Mode::Outer;
    }

    // $ident
    if let Some(b) = cur.peek() {
        if is_ident_head(b) {
            let s = cur.i;
            cur.bump();
            while let Some(nb) = cur.peek() {
                if is_ident_tail(nb) {
                    cur.bump();
                } else {
                    break;
                }
            }
            out.push(Token::ident(Span::new(s, cur.i)));
            return Mode::Outer;
        }
    }

    // 其他：回 Outer（让它把字符并入 Text）
    Mode::Outer
}

/// ${ ... } 体内扫描到 `}` 为止；中途可切换到其它子状态
pub fn scan_inside_brace(cur: &mut Cursor, out: &mut Vec<Token>) -> Mode {
    // 先读左花括号
    bump_and_push(cur, out, Token::lbrace, 1);

    let mut text_start: Option<usize> = None;
    let mut phase = InBracePhase::ExpectHead;

    while let Some(b) = cur.peek() {
        // 读到右花括号，结束当前 brace
        if b == b'}' {
            push_text_if_any(&mut text_start, cur, out);
            bump_and_push(cur, out, Token::rbrace, 1);
            return Mode::Outer;
        }

        // 双字符优先：##
        if cur.starts_with(b'#', b'#') && matches!(phase, InBracePhase::AfterHead) {
            // 对于双##，只处理修剪模式，否则应交由下方的单#处理
            push_text_if_any(&mut text_start, cur, out);
            bump_and_push(cur, out, Token::double_hash, 2);
            read_trim_pattern(cur, out);
            continue;
        }
        // 双字符优先：%%
        if cur.starts_with(b'%', b'%') {
            push_text_if_any(&mut text_start, cur, out);
            bump_and_push(cur, out, Token::double_percent, 2);
            read_trim_pattern(cur, out);
            continue;
        }
        // 双字符优先：//
        if cur.starts_with(b'/', b'/') {
            push_text_if_any(&mut text_start, cur, out);
            bump_and_push(cur, out, Token::double_slash, 2);
            continue;
        }

        // SignedInt: [+-]?[0-9]+
        if let Some(nb) = cur.peek() {
            let is_num_head =
                nb.is_ascii_digit() || (matches!(nb, b'+' | b'-') && cur.i + 1 < cur.len() && cur.bytes[cur.i + 1].is_ascii_digit());
            if is_num_head {
                push_text_if_any(&mut text_start, cur, out);
                let s = cur.i;
                if matches!(nb, b'+' | b'-') {
                    cur.bump();
                }
                while let Some(d) = cur.peek() {
                    if d.is_ascii_digit() {
                        cur.bump();
                    } else {
                        break;
                    }
                }
                out.push(Token::signed_int(Span::new(s, cur.i)));
                continue;
            }
        }

        // 单字符分隔/符号 + 状态转移
        match b {
            b'#' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::hash, 1);
                if matches!(phase, InBracePhase::AfterHead) {
                    read_trim_pattern(cur, out);
                } else if Some(b'#') == cur.peek() {
                    // 如果在 head 之前读到 #，则认为是 SpecialParam 的长度
                    bump_and_push(cur, out, Token::special_param, 1);
                    // 进入 AfterHead 状态，等待下一个字符
                    phase = InBracePhase::AfterHead;
                }
                continue;
            }
            b'%' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::percent, 1);
                read_trim_pattern(cur, out);
                continue;
            }
            b'/' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::slash, 1);
                continue;
            }
            b':' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::colon, 1);
                continue;
            }
            b'(' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::lparen, 1);
                return Mode::Flags;
            }
            b')' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::rparen, 1);
                continue;
            }
            b'[' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::lbracket, 1);
                return Mode::Subscript;
            }
            b']' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::rbracket, 1);
                continue;
            }
            b',' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::comma, 1);
                continue;
            }
            b'+' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::plus, 1);
                continue;
            }
            b'-' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::minus, 1);
                continue;
            }
            b'=' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::equals, 1);
                continue;
            }
            b'?' => {
                push_text_if_any(&mut text_start, cur, out);
                bump_and_push(cur, out, Token::question, 1);
                continue;
            }
            _ => {}
        }

        // SpecialParam（花括号内）
        if matches!(b, b'*' | b'@' | b'#' | b'?' | b'-' | b'$' | b'!' | b'_') || b.is_ascii_digit() {
            push_text_if_any(&mut text_start, cur, out);
            bump_and_push(cur, out, Token::special_param, 1);
            phase = InBracePhase::AfterHead; // 读到 SpecialParam 后，认为已读到 head
            continue;
        }

        // Ident
        if is_ident_head(b) {
            push_text_if_any(&mut text_start, cur, out);
            let s = cur.i;
            cur.bump();
            while let Some(nb) = cur.peek() {
                if is_ident_tail(nb) {
                    cur.bump();
                } else {
                    break;
                }
            }
            out.push(Token::ident(Span::new(s, cur.i)));
            phase = InBracePhase::AfterHead; // 读到 Ident 后，认为已读到 head
            continue;
        }

        // 其他：聚为 Text（以后在 Pattern/Sep 里细分）
        if text_start.is_none() {
            text_start = Some(cur.i);
        }
        cur.bump();
    }

    // 未闭合：尽量收尾
    push_text_if_any(&mut text_start, cur, out);
    Mode::Outer
}

/// 读取修剪用的 pattern（用于 %/%%/#/## 之后）。
/// 结束边界：`}`、`:`、`/`、`#`、`%` —— 不吃边界，只吐中间为一个 Pattern。
fn read_trim_pattern(cur: &mut super::util::Cursor, out: &mut Vec<crate::Token>) {
    use crate::{Span, Token, TokenKind};
    if cur.eof() {
        return;
    }

    let start = cur.i;
    while let Some(b) = cur.peek() {
        match b {
            b'}' | b':' | b'/' | b'#' | b'%' => break, // 遇边界停止（不消费）
            _ => {
                cur.bump();
            }
        }
    }
    // 空 pattern 不吐
    if cur.i > start {
        out.push(Token::new(TokenKind::Pattern, Span::new(start, cur.i)));
    }
}

/// FLAGS：占位；后续实现 flag 解析、(s:..:)/(j:..:) 进入 Sep
pub fn scan_flags(cur: &mut Cursor, _out: &mut Vec<Token>) -> Mode {
    let _ = cur.bump();
    Mode::Flags
}

/// SEP：占位；后续实现：读到下一个不转义的 ':'，产出 SepString
pub fn scan_sep(cur: &mut Cursor, _out: &mut Vec<Token>) -> Mode {
    let _ = cur.bump();
    Mode::Flags
}

/// PATTERN：占位；后续实现 /pat/ 与 #pat/%pat 的结束边界
pub fn scan_pattern(cur: &mut Cursor, _out: &mut Vec<Token>) -> Mode {
    let _ = cur.bump();
    Mode::Expansion
}

/// SUBSCRIPT：占位；后续实现 [n] / [n,m]，遇 ']' 返回 Expansion
pub fn scan_subscript(cur: &mut Cursor, _out: &mut Vec<Token>) -> Mode {
    let _ = cur.bump();
    Mode::Subscript
}
