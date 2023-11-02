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
        cx.set_font("18px monospace");

        let mut height = 0f64;
        let lines = lines
            .map(|line| {
                let chars = line
                    .chars()
                    .map(|c| {
                        let text_metrics = cx.measure_text(&c.to_string()).unwrap();
                        height = height.max(
                            text_metrics.actual_bounding_box_ascent()
                                + text_metrics.actual_bounding_box_descent(),
                        );
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
}
