use lazy_static::lazy_static;
use ropey::Rope;
use std::{mem, rc::Rc};
use tree_sitter_c2rust::{InputEdit, Node, Parser, Point, Query, QueryCursor, Range, Tree};
use tree_sitter_rust::HIGHLIGHT_QUERY;

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
        let idx = self.rope.line_to_char(line) + col;
        self.rope.insert(idx, text);

        let byte_idx = self.rope.char_to_byte(idx);
        let edit = InputEdit {
            start_byte: byte_idx,
            old_end_byte: byte_idx,
            new_end_byte: byte_idx + text.len(),
            start_position: Point::new(line, col),
            old_end_position: Point::new(line, col),
            new_end_position: Point::new(line, col + text.len()),
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
        let spans = self.spans();
        let mut lines = Vec::new();
        let mut line = Vec::new();
        let mut line_idx = 0;
        for span in spans {
            let span_line_idx = self.rope.byte_to_line(span.start);
            if span_line_idx != line_idx {
                line_idx = span_line_idx;
                lines.push(mem::take(&mut line));
            }
            line.push(span);
        }
        lines.push(line);
        lines
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

    fn spans(&self) -> Vec<Span> {
        let highlights = self.highlights();
        let mut iter = self.rope.bytes().enumerate().peekable();
        let mut spans = Vec::new();
        let mut start = 0;

        while let Some((idx, _c)) = iter.next() {
            for highlight in highlights.iter() {
                if highlight.range.start_byte <= idx && highlight.range.end_byte >= idx {
                    if start < idx {
                        spans.push(Span {
                            kind: None,
                            text: Rc::new(self.rope.slice(start..idx).to_string()),
                            start: start,
                            end: idx,
                        })
                    }

                    let mut end = idx;
                    'a: while let Some((next_idx, _)) = iter.peek() {
                        if highlight.range.start_byte <= *next_idx
                            && highlight.range.end_byte >= *next_idx
                        {
                            iter.next();
                        } else {
                            end = *next_idx - 1;
                            break 'a;
                        }
                    }
                    spans.push(Span {
                        kind: Some(Rc::new(highlight.kind.clone())),
                        text: Rc::new(self.rope.slice(idx..end).to_string()),
                        start: idx,
                        end,
                    });
                    start = end;
                }
            }
        }

        spans.push(Span {
            kind: None,
            text: Rc::new(self.rope.slice(start..).to_string()),
            start: start,
            end: self.rope.len_bytes(),
        });
        spans
    }
}
