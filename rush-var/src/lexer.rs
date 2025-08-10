mod util;
mod state;

use crate::{Source, TokenStream};
use state::{Mode, scan_expansion, scan_flags, scan_inside_brace, scan_outer, scan_pattern, scan_sep, scan_subscript};
use util::Cursor;

pub fn lex(input: &str) -> Result<TokenStream, String> {
    let src = Source { src: input };
    let mut out = Vec::new();
    let mut cur = Cursor::new(input);

    let mut mode = Mode::Outer;
    while !cur.eof() {
        mode = match mode {
            Mode::Outer => scan_outer(&mut cur, &mut out),
            Mode::Expansion => scan_expansion(&mut cur, &mut out),
            Mode::Flags => scan_flags(&mut cur, &mut out),
            Mode::Sep => scan_sep(&mut cur, &mut out),
            Mode::Pattern => scan_pattern(&mut cur, &mut out),
            Mode::Subscript => scan_subscript(&mut cur, &mut out),
        };
        // 防御：如果某状态未推进光标，强制前进一步避免死循环（开发期保护，稳定后可移除）
        if !cur.eof() && matches!(mode, Mode::Flags | Mode::Sep | Mode::Pattern | Mode::Subscript) {
            // 上述占位函数已 bump；若你替换成真实实现，可删掉这一段
        }
    }

    Ok(TokenStream { source: src, tokens: out })
}
