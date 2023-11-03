use crate::{use_buffer, Buffer};
use dioxus::prelude::Scope;
use dioxus_signals::{use_signal, Signal, Write};
use std::cell::Ref;
use tree_sitter_c2rust::Point;

pub fn use_editor<'a, T>(cx: Scope<T>, make_text: impl FnOnce() -> &'a str) -> UseEditor {
    let buffer = use_buffer(cx, make_text);
    let cursor = use_signal(cx, || Point::new(0, 0));
    let is_focused = use_signal(cx, || false);

    UseEditor {
        buffer,
        cursor,
        is_focused,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct UseEditor {
    buffer: Signal<Buffer>,
    cursor: Signal<Point>,
    is_focused: Signal<bool>,
}

impl UseEditor {
    pub fn buffer(&self) -> Ref<Buffer> {
        self.buffer.read()
    }

    pub fn buffer_mut(&self) -> Write<Buffer> {
        self.buffer.write()
    }

    pub fn cursor(&self) -> Point {
        *self.cursor.read()
    }

    pub fn cursor_mut(&self) -> Write<Point> {
        self.cursor.write()
    }

    pub fn is_focused(&self) -> bool {
        *self.is_focused.read()
    }

    pub fn focus(&self) {
        self.is_focused.set(true)
    }

    pub fn blur(&self) {
        self.is_focused.set(false)
    }

    pub fn insert(&self, text: &str) {
        let mut cursor_ref = self.cursor.write();
        self.buffer
            .write()
            .insert(cursor_ref.row, cursor_ref.column, &text);
        cursor_ref.column += text.len();
    }
}
