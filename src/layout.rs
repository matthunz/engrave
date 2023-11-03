use std::collections::HashMap;

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
    pub y: f64,
}

pub struct Layout {
    lines: Vec<Line>,
    canvas: HtmlCanvasElement,
    char_widths: HashMap<char, f64>,
    font_size: f64,
    line_height: f64,
}

impl Layout {
    pub fn new(font_size: f64, line_height: f64) -> Self {
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
            char_widths: HashMap::new(),
            font_size,
            line_height,
        }
    }

    pub fn measure<'a>(&mut self, lines: impl Iterator<Item = RopeSlice<'a>>) {
        let cx_object = self.canvas.get_context("2d").unwrap().unwrap();
        let cx = cx_object.unchecked_ref::<CanvasRenderingContext2d>();
        cx.set_font(&format!("{}px monospace", self.font_size));

        self.lines = lines
            .enumerate()
            .map(|(idx, line)| {
                let y = idx as f64 * self.line_height;
                let mut current_x = 0.;
                let chars = line
                    .chars()
                    .map(|c| {
                        let width = if let Some(width) = self.char_widths.get(&c) {
                            *width
                        } else {
                            let text_metrics = cx.measure_text(&c.to_string()).unwrap();
                            self.char_widths.insert(c, text_metrics.width());
                            text_metrics.width()
                        };

                        let x = current_x;
                        current_x += width;

                        Char { c, width, x, y }
                    })
                    .collect();

                Line {
                    chars,
                    height: self.line_height,
                    y,
                }
            })
            .collect();
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn pos(&self, point: Point) -> Option<[f64; 2]> {
        let line_char = &self.lines.get(point.row)?.chars.get(point.column)?;
        Some([line_char.x, line_char.y])
    }

    pub fn line(&self, y: f64) -> Option<usize>{
        let mut current_y = 0.;
        for (idx, line) in self.lines.iter().enumerate() {
           current_y += line.height;
            if current_y >= y {
                return Some(idx);
            }
        }
        None
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
