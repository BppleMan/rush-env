use rush_var::{Source, Span};

#[test]
fn source_slice_and_first_char() {
    let s = Source { src: "ab\ncd" };
    assert_eq!(s.slice(Span::new(0, 2)), "ab");
    assert_eq!(s.first_char(Span::new(2, 3)), Some('\n'));
}

#[test]
fn source_parse_i64_ok_err() {
    let s = Source { src: "x-42y" };
    assert_eq!(s.parse_i64(Span::new(1, 4)).unwrap(), -42);
    assert!(s.parse_i64(Span::new(0, 1)).is_err()); // "x" -> Err
}
