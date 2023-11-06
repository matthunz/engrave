mod buffer;
pub use buffer::{use_buffer, Buffer};

mod editor;
pub use editor::{Builder, Editor, UseEditor};

pub mod language;
pub use language::Language;

mod layout;

mod span;
pub use span::Span;

mod use_query;
pub use use_query::use_query_signal;

pub use tree_sitter_c2rust::Point;

#[derive(Clone, Copy, Default)]
pub struct Range {
    pub start: Point,
    pub end: Point,
}

impl Range {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}
