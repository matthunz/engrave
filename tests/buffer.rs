use editor::{Buffer, Span};
use std::rc::Rc;

#[test]
fn it_works() {
    let buffer = Buffer::new("\"Hello World!\"");
    assert_eq!(
        buffer.lines()[0],
        &[Span {
            kind: Some(Rc::new(String::from("string_literal"))),
            text: Rc::new(String::from("\"Hello World!\"")),
            start: 0,
            end: "\"Hello World!\"".len()
        }]
    )
}
