mod buffer;
pub use buffer::{use_buffer, Buffer};

mod editor;
pub use editor::{Builder, Editor, UseEditor};

pub mod language;
pub use language::{use_language, Language};

mod layout;

mod span;
pub use span::Span;

mod use_highlights;
pub use use_highlights::{use_highlights, Highlight};

mod use_query;
pub use use_query::{use_query, use_query_signal};

pub use tree_sitter_c2rust::Point;

#[derive(Clone, Copy, Default, Debug)]
pub struct Range {
    pub start: Point,
    pub end: Point,
}

impl Range {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}
