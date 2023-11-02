use ropey::RopeSlice;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Char {
    pub c: char,
    pub width: f64,
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
    pub fn new<'a>(lines: impl Iterator<Item = RopeSlice<'a>>) -> Self {
        let elem = window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap();
        let canvas: HtmlCanvasElement = elem.unchecked_into();

        let cx_object = canvas.get_context("2d").unwrap().unwrap();
        let cx = cx_object.unchecked_ref::<CanvasRenderingContext2d>();
        cx.set_font("16px monospace");

        let height = 26.;
        let lines = lines
            .map(|line| {
                let chars = line
                    .chars()
                    .map(|c| {
                        let text_metrics = cx.measure_text(&c.to_string()).unwrap();

                        Char {
                            c,
                            width: text_metrics.width(),
                        }
                    })
                    .collect();
                Line { chars, height }
            })
            .collect();

        Self { lines, canvas }
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
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
