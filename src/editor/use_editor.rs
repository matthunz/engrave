use crate::{language, use_buffer, use_highlights, Buffer, Language, Range, Span};
use dioxus::prelude::{use_context_provider, Scope};
use dioxus_lazy::{
    lazy::{self, Values},
    Direction, UseLazyAsync, UseList,
};
use dioxus_resize_observer::{use_resize, Rect};
use dioxus_signals::{use_signal, Signal, Write};
use std::cell::Ref;

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

    pub fn use_editor<'a, 's, T>(
        self,
        cx: Scope<'a, T>,
        make_text: impl FnOnce() -> &'s str,
    ) -> UseEditor {
        let language = self.language;
        use_context_provider(cx, || Signal::new(language));

        let buffer = use_buffer(cx, language.tree_sitter, make_text);
        let is_focused = use_signal(cx, || false);
        let selections = use_signal(cx, || Vec::new());
        let highlights = use_highlights(cx, buffer);
        let list = UseList::builder()
            .direction(Direction::Row)
            .size(self.height)
            .item_size(self.line_height)
            .len(buffer().rope.len_lines())
            .use_list(
                cx,
                lazy::from_async_range_fn(move |range, is_rev| async move {
                    let mut lines = buffer().lines(range, &highlights());
                    if is_rev {
                        lines.reverse();
                    }
                    lines
                }),
            );
        let container_size = use_resize(cx, list.mounted);

        UseEditor {
            buffer,
            is_focused,

            container_size,
            list,
            selections,
            height: self.height,
            line_height: self.line_height,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct UseEditor {
    pub buffer: Signal<Buffer>,
    is_focused: Signal<bool>,
    pub container_size: Signal<Option<Rect>>,
    pub list: UseList<UseLazyAsync<Vec<Span>>>,
    pub selections: Signal<Vec<Range>>,
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
        *self.list.scroll_range.scroll.read()
    }

    pub fn insert(&self, text: &str) {
        let cursor_ref = &mut self.selections.write()[0].start;
        self.buffer
            .write()
            .insert(cursor_ref.row, cursor_ref.column, text);
        cursor_ref.column += text.len();
        cursor_ref.row += text.lines().count() - 1;
        self.list.lazy.refresh();
    }
}
