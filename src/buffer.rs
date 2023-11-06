use crate::{
    use_query::{use_language, use_query},
    Span,
};
use dioxus::prelude::Scope;
use dioxus_signals::{use_signal, Signal};
use ropey::{Rope, RopeSlice};
use std::mem;
use tree_sitter_c2rust::{InputEdit, Language, Node, Parser, Point, Range, TextProvider, Tree};

pub fn use_buffer<'a, T>(
    cx: Scope<T>,
    language: Language,
    make_text: impl FnOnce() -> &'a str,
) -> Signal<Buffer> {
    use_signal(cx, || Buffer::new(language, make_text()))
}

pub fn use_highlights<T>(cx: Scope<T>, buffer: Signal<Buffer>) -> Signal<Vec<Item>> {
    let language = use_language(cx);
    let highlights = use_signal(cx, || Vec::new());
    use_query(cx, language().highlight_query, buffer, move |matches| {
        let items = matches
            .flat_map(|mat| {
                mat.captures.iter().map(|capture| {
                    let range = capture.node.range();
                    let kind = capture.node.kind().to_owned();
                    Item { range, kind }
                })
            })
            .collect();
        highlights.set(items);
    });
    highlights
}

#[derive(Debug)]
pub struct Item {
    kind: String,
    range: Range,
}

pub struct Buffer {
    pub rope: Rope,
    parser: Parser,
    pub tree: Tree,
}

impl Buffer {
    pub fn new(language: Language, text: &str) -> Self {
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();
        let tree = parser.parse(text, None).unwrap();

        Self {
            rope: Rope::from_str(text),
            parser,
            tree,
        }
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn insert(&mut self, line: usize, col: usize, text: &str) -> Tree {
        let char_idx = self.rope.line_to_char(line) + col;
        let idx = self.rope.char_to_byte(char_idx);
        self.rope.insert(char_idx, text);

        let edit = InputEdit {
            start_byte: idx,
            old_end_byte: idx,
            new_end_byte: idx + text.len(),
            start_position: Point::new(line, col),
            old_end_position: Point::new(line, col),
            new_end_position: Point::new(
                line + text.lines().count() - 1,
                col + text.chars().count(),
            ),
        };
        self.tree.edit(&edit);

        let tree = self
            .parser
            .parse_with(
                &mut |idx, _| {
                    self.rope
                        .get_chunk_at_byte(idx)
                        .map(|(chunk, _, _, _)| chunk)
                        .unwrap_or_default()
                },
                None, // Some(&self.tree),
            )
            .unwrap();
        mem::replace(&mut self.tree, tree)
    }

    pub fn lines(&self, range: std::ops::Range<usize>, highlights: &[Item]) -> Vec<Vec<Span>> {
        self.rope
            .lines_at(range.start)
            .take(range.end - range.start)
            .enumerate()
            .map(|(idx, line)| {
                let idx = idx + range.start;
                let mut spans = Vec::new();
                let mut iter = line.chars().enumerate().peekable();
                let mut start = 0;

                while let Some((col, _)) = iter.next() {
                    for highlight in highlights {
                        let start_point = highlight.range.start_point;
                        let end_point = highlight.range.end_point;

                        if start_point.row <= idx
                            && end_point.row >= idx
                            && start_point.column <= col
                            && end_point.column > col
                        {
                            let mut end = None;
                            while let Some((next_col, _)) = iter.peek() {
                                if start_point.column <= *next_col && end_point.column > *next_col {
                                    iter.next();
                                } else {
                                    end = Some(*next_col);
                                    break;
                                }
                            }

                            if let Some(end) = end {
                                if start < col {
                                    spans.push(Span::from_text(line.slice(start..col).to_string()));
                                }
                                start = end;

                                spans.push(Span::from_kind(
                                    &*highlight.kind,
                                    line.slice(col..end).to_string(),
                                ))
                            }
                        }
                    }
                }

                spans.push(Span::from_text(line.slice(start..).to_string()));
                spans
            })
            .collect()
    }
}

pub struct Iter<'a> {
    chunks: ropey::iter::Chunks<'a>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(str::as_bytes)
    }
}

pub struct RopeProvider<'a> {
    pub slice: RopeSlice<'a>,
}

impl<'a> TextProvider<'a> for RopeProvider<'a> {
    type I = Iter<'a>;

    fn text(&mut self, node: Node) -> Self::I {
        let len = self.slice.len_bytes();
        let start = node.end_byte().min(len);
        let end = node.end_byte().min(len);
        let chunks = self.slice.byte_slice(start..end).chunks();
        Iter { chunks }
    }
}
