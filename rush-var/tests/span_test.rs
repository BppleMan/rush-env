use rush_var::Span;

#[test]
fn span_len_and_empty() {
    let sp = Span::new(3, 8);
    assert_eq!(sp.len(), 5);
    let at = Span::empty_at(10);
    assert_eq!(at.start, 10);
    assert_eq!(at.end, 10);
    assert_eq!(at.len(), 0);
}
