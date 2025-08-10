use rush_var::{lex, pretty_tokens};

macro_rules! snap_case {
    ($name:literal, $input:expr) => {{
        let stream = lex($input).expect("lexing should succeed");
        let dump = pretty_tokens(&stream);
        let body = format!("input: {}\n{}", $input, dump);
        insta::assert_snapshot!($name, body);
    }};
}

#[test]
fn lexer_basic() {
    snap_case!("text_only", "hello world");
    snap_case!("simple_dollar_ident", "Hello $USER!");
    snap_case!("braced_ident", "Path: ${HOME}/bin");
}

#[test]
fn lexer_defaulting_family() {
    snap_case!("colon_minus", "${name:-def}");
    snap_case!("colon_plus", "${name:+X}");
    snap_case!("colon_question", "${name:?ERR}");
    snap_case!("colon_equals", "${name:=val}");
}

#[test]
fn lexer_trim_and_replace() {
    snap_case!("double_hash_star_slash", "${v##*/}");
    snap_case!("double_percent_suffix", "${file%%.bak}");
    snap_case!("replace_first", "${v/a/b}");
    snap_case!("replace_global", "${v//a/b}");
}

#[test]
fn lexer_slices() {
    snap_case!("offset_only", "${v:2}");
    snap_case!("offset_len", "${v:2:3}");
    snap_case!("signed_offsets", "${v:+2:-3}");
}

#[test]
fn lexer_length_and_nested() {
    snap_case!("length_of_ident", "${#HOME}");
    snap_case!("slice_with_length", "${v:0:${#v}}");
}

#[test]
fn lexer_double_hash() {
    snap_case!("double_hash", "${##}");
}

#[test]
fn lexer_special_params() {
    snap_case!("specials_short_form", "args=$@ idx=$9 pid=$$");
}

#[test]
fn lexer_mixed_line() {
    snap_case!("mixed_text_and_expansions", "Hi $USER, home=${HOME:-/tmp}, file=${v##*/}");
}

#[test]
fn lexer_unclosed_brace_recovery() {
    snap_case!("unclosed_brace", "${HOME/path");
}

// ===== 下面三组留作 TODO，等实现对应状态后去掉 #[ignore] =====

#[test]
#[ignore = "TODO: SUBSCRIPT 状态（[n]/[n,m]）"]
fn lexer_subscript_todo() {
    snap_case!("subscript_index_and_range", "${arr[2,4]}");
}

#[test]
#[ignore = "TODO: FLAGS + SEP（(s:SEP:)/(j:SEP:)）"]
fn lexer_flags_sep_todo() {
    snap_case!("flags_split", "${(s:|:)S}");
    snap_case!("flags_join", "${(j:-)A}");
}

#[test]
#[ignore = "TODO: PATTERN 专用读取（/pat/ 与 #pat/%pat）"]
fn lexer_pattern_todo() {
    snap_case!("pattern_replace_end_slash", "${v/%foo/bar/}");
    snap_case!("pattern_replace_anchor", "${v/#ab/XY}");
}
