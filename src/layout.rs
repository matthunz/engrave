use ropey::RopeSlice;
use tree_sitter_c2rust::Point;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Char {
    pub c: char,
    pub width: f64,
    pub x: f64,
    pub y: f64,
}

pub struct Line {
    pub chars: Vec<Char>,
    pub height: f64,
}

pub struct Layout {
    lines: Vec<Line>,
    canvas: HtmlCanvasElement,
}

impl Layout {
    pub fn new() -> Self {
        let elem = window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap();
        let canvas: HtmlCanvasElement = elem.unchecked_into();

        Self {
            lines: Vec::new(),
            canvas,
        }
    }

    pub fn measure<'a>(&mut self, lines: impl Iterator<Item = RopeSlice<'a>>) {
        let cx_object = self.canvas.get_context("2d").unwrap().unwrap();
        let cx = cx_object.unchecked_ref::<CanvasRenderingContext2d>();
        cx.set_font("16px monospace");

        let height = 26.;
        self.lines = lines
            .enumerate()
            .map(|(idx, line)| {
                let mut current_x = 0.;
                let chars = line
                    .chars()
                    .map(|c| {
                        let text_metrics = cx.measure_text(&c.to_string()).unwrap();
                        let x = current_x;
                        current_x += text_metrics.width();

                        Char {
                            c,
                            width: text_metrics.width(),
                            x: x,
                            y: idx as f64 * height,
                        }
                    })
                    .collect();
                Line { chars, height }
            })
            .collect();
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn pos(&self, point: Point) -> [f64; 2] {
        let line_char = &self.lines[point.row].chars[point.column];
        [line_char.x, line_char.y]
    }

    pub fn target(&self, x: f64, y: f64) -> Option<(usize, Option<usize>)> {
        let mut current_y = 0.;
        for (line_idx, line) in self.lines.iter().enumerate() {
            let bottom = current_y + line.height;
            current_y = bottom;

            if bottom >= y {
                let mut current_x = 0.;

                for (col_idx, line_char) in line.chars.iter().enumerate() {
                    let right = current_x + line_char.width;
                    current_x = right;

                    if right >= x {
                        return Some((line_idx, Some(col_idx)));
                    }
                }

                return Some((line_idx, None));
            }
        }
        None
    }
}
