use dioxus::prelude::Scope;
use dioxus_signals::{use_signal, Signal};
use lazy_static::lazy_static;
use ropey::Rope;
use std::{mem, rc::Rc};
use tree_sitter_c2rust::{InputEdit, Node, Parser, Point, Query, QueryCursor, Range, Tree};
use tree_sitter_rust::HIGHLIGHT_QUERY;

pub fn use_buffer<'a, T>(cx: Scope<T>, make_text: impl FnOnce() -> &'a str) -> Signal<Buffer> {
    use_signal(cx, || Buffer::new(make_text()))
}

#[derive(Clone, Debug, Eq)]
pub struct Span {
    pub kind: Option<Rc<String>>,
    pub text: Rc<String>,
    pub start: usize,
    pub end: usize,
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && Rc::ptr_eq(&self.text, &other.text)
            && self.start == other.start
            && self.end == other.end
    }
}

#[derive(Debug)]
struct Item {
    kind: String,
    range: Range,
}

pub struct Buffer {
    pub rope: Rope,
    parser: Parser,
    tree: Tree,
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
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
                        .get_chunks_at_byte(idx)
                        .and_then(|mut chunk| chunk.0.next())
                        .unwrap_or_default()
                },
                Some(&self.tree),
            )
            .unwrap();
        mem::replace(&mut self.tree, tree)
    }

    pub fn lines(&self) -> Vec<Vec<Span>> {
        let highlights = self.highlights();

        self.rope
            .lines()
            .enumerate()
            .map(|(idx, line)| {
                let mut spans = Vec::new();
                let mut iter = line.chars().enumerate().peekable();
                let mut start = 0;

                while let Some((col, _c)) = iter.next() {
                    for highlight in &highlights {
                        let start_point = highlight.range.start_point;
                        let end_point = highlight.range.end_point;

                        if start_point.row <= idx && end_point.row >= idx {
                            let mut end = None;
                            if start_point.column <= col && end_point.column >= col {
                                while let Some((next_col, _next_c)) = iter.peek() {
                                    if start_point.column <= *next_col
                                        && end_point.column >= *next_col
                                    {
                                        iter.next();
                                    } else {
                                        end = Some(*next_col);
                                        break;
                                    }
                                }
                            }
                            if let Some(end) = end {
                                if start < col {
                                    spans.push(Span {
                                        kind: None,
                                        text: Rc::new(line.slice(start..col).to_string()),
                                        start: 0,
                                        end: 0,
                                    });
                                }
                                start = end - 1;

                                spans.push(Span {
                                    kind: Some(Rc::new(highlight.kind.clone())),
                                    text: Rc::new(line.slice(col..end - 1).to_string()),
                                    start: 0,
                                    end: 0,
                                })
                            }
                        }
                    }
                }

                spans.push(Span {
                    kind: None,
                    text: Rc::new(line.slice(start..).to_string()),
                    start: 0,
                    end: 0,
                });
                spans
            })
            .collect()
    }

    fn highlights(&self) -> Vec<Item> {
        lazy_static! {
            static ref QUERY: Query =
                Query::new(tree_sitter_rust::language(), HIGHLIGHT_QUERY).unwrap();
        }

        let mut query_cursor = QueryCursor::new();
        let rope = &self.rope;
        let matches = query_cursor.matches(&QUERY, self.tree.root_node(), move |node: Node| {
            rope.get_byte_slice(node.start_byte()..node.end_byte())
                .map(|slice| slice.chunks().map(move |chunk| chunk.as_bytes()))
                .into_iter()
                .flatten()
        });
        matches
            .flat_map(|mat| {
                mat.captures.iter().map(|capture| {
                    let range = capture.node.range();
                    let kind = capture.node.kind().to_owned();
                    Item { range, kind }
                })
            })
            .collect()
    }
}
