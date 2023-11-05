use crate::{language, use_buffer, use_query, Buffer, Language, Span};
use dioxus::prelude::{use_context_provider, use_effect, Scope};
use dioxus_lazy::{factory, Direction, UseList};
use dioxus_resize_observer::{use_resize, Rect};
use dioxus_signals::{use_signal, Signal, Write};
use std::cell::Ref;
use tree_sitter_c2rust::{Point, Query};

#[derive(Clone, Copy, PartialEq)]
pub struct Builder {
    font_size: f64,
    height: f64,
    line_height: f64,
    language: Language,
}

impl Builder {
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    pub fn line_height(mut self, line_height: f64) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    pub fn use_editor<'a, T>(self, cx: Scope<T>, make_text: impl FnOnce() -> &'a str) -> UseEditor {
        let language = self.language;
        let language_signal = use_context_provider(cx, || Signal::new(language));
        use_effect(cx, &language, |lang| {
            language_signal.set(lang);
            async {}
        });

        let buffer = use_buffer(cx, language.tree_sitter, make_text);
        let cursor = use_signal(cx, || Point::new(0, 0));
        let is_focused = use_signal(cx, || false);
        let query = use_query(cx, language.highlight_query);

        let list = UseList::builder()
            .direction(Direction::Row)
            .size(self.height)
            .item_size(self.line_height)
            .len(buffer().rope.len_lines())
            .use_list(
                cx,
                factory::from_range_fn(move |range, is_rev| async move {
                    let mut lines = buffer().lines(&query(), range);
                    if is_rev {
                        lines.reverse();
                    }
                    lines
                }),
            );

        let container_size = use_resize(cx, list.mounted);

        UseEditor {
            buffer,
            cursor,
            is_focused,
            query,
            container_size,
            list,
            height: self.height,
            line_height: self.line_height,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct UseEditor {
    buffer: Signal<Buffer>,
    cursor: Signal<Point>,
    is_focused: Signal<bool>,
    pub(crate) query: Signal<Query>,
    pub container_size: Signal<Option<Rect>>,
    pub list: UseList<Vec<Span>>,
    pub height: f64,
    pub line_height: f64,
}

impl UseEditor {
    pub fn builder() -> Builder {
        Builder {
            font_size: 16.,
            height: 400.,
            line_height: 24.,
            language: language::rust(),
        }
    }

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
        *self.list.scroll.read()
    }

    pub fn insert(&self, text: &str) {
        let mut cursor_ref = self.cursor.write();
        self.buffer
            .write()
            .insert(cursor_ref.row, cursor_ref.column, text);
        cursor_ref.column += text.len();
    }
}
