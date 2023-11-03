use crate::{use_buffer, use_query, Buffer, Language};
use dioxus::prelude::{use_context_provider, use_effect, Scope};
use dioxus_resize_observer::use_size;
use dioxus_signals::{use_signal, Signal, Write};
use dioxus_use_mounted::{use_mounted, UseMounted};
use std::cell::Ref;
use tree_sitter_c2rust::{Point, Query};

pub fn use_editor<'a, T>(
    cx: Scope<T>,
    language: Language,
    make_text: impl FnOnce() -> &'a str,
) -> UseEditor {
    let language_signal = use_context_provider(cx, || Signal::new(language));
    use_effect(cx, &language, |lang| {
        language_signal.set(lang);
        async {}
    });

    let buffer = use_buffer(cx, language.tree_sitter, make_text);
    let cursor = use_signal(cx, || Point::new(0, 0));
    let is_focused = use_signal(cx, || false);
    let query = use_query(cx, language.highlight_query);
    let scroll = use_signal(cx, || 0);

    let container_ref = use_mounted(cx);
    let container_size = use_size(cx, container_ref);

    UseEditor {
        buffer,
        cursor,
        is_focused,
        query,
        container_ref,
        container_height: container_size.height(),
        scroll,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct UseEditor {
    buffer: Signal<Buffer>,
    cursor: Signal<Point>,
    is_focused: Signal<bool>,
    pub(crate) query: Signal<Query>,
    pub container_ref: UseMounted,
    pub container_height: f64,
    scroll: Signal<i32>,
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

    pub fn scroll(&self) -> i32 {
        *self.scroll.read()
    }

    pub fn set_scroll(&self, scroll: i32) {
        self.scroll.set(scroll)
    }

    pub fn insert(&self, text: &str) {
        let mut cursor_ref = self.cursor.write();
        self.buffer
            .write()
            .insert(cursor_ref.row, cursor_ref.column, &text);
        cursor_ref.column += text.len();
    }
}
