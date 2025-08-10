use rush_var::{lex, pretty_tokens};

macro_rules! snap {
    ($name:literal, $input:expr) => {{
        let ts = lex($input).unwrap();
        let dump = format!("input: {}\n{}", $input, pretty_tokens(&ts));
        insta::assert_snapshot!($name, dump);
    }};
}

#[test]
fn signed_int_heads_and_unclosed() {
    snap!("slice_pos_neg", "${v:+2:-3}"); // +2 / -3 → SignedInt
    snap!("unclosed_brace_tail", "${foo"); // 未闭合路径
}

#[test]
fn special_params_both_places() {
    snap!("special_short", "a=$@ b=$9 c=$$");
    snap!("special_in_brace", "${*}${9}${$}${_}");
}

#[test]
fn double_ops_priority() {
    snap!("double_hash", "${v##x}");
    snap!("double_percent", "${v%%x}");
    snap!("double_slash", "${v//a/b}");
}

#[test]
fn single_char_ops_and_comma() {
    snap!("single_ops_and_comma", "${a:1,2}${b:-x}${c:+y}${d:?e}${e:=z}");
}
