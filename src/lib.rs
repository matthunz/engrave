use ropey::Rope;
use tree_sitter::{InputEdit, Parser, Point, Range, Tree};

pub struct Editor {
    rope: Rope,
    parser: Parser,
    tree: Tree,
}

impl Editor {
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

    pub fn insert(
        &mut self,
        line: usize,
        col: usize,
        text: &str,
    ) -> impl ExactSizeIterator<Item = Range> {
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
                &mut |idx, _| self.rope.chunks_at_byte(idx).0.next().unwrap_or_default(),
                Some(&self.tree),
            )
            .unwrap();
        let changed_ranges = self.tree.changed_ranges(&tree);
        self.tree = tree;

        for node in self.tree.root_node().children(&mut self.tree.walk()) {
            dbg!(node);
        }

        changed_ranges
    }
}

#[cfg(test)]
mod tests {
    use crate::Editor;

    #[test]
    fn it_works() {
        let mut editor = Editor::new("fn main() {}");
        for range in editor.insert(0, 0, "fn f() {}") {
            dbg!(range);
        }
    }
}
