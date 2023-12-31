use dioxus::prelude::{use_context, Scope};
use dioxus_signals::Signal;

pub fn use_language<T>(cx: Scope<T>) -> Signal<Language> {
    *use_context(cx).unwrap()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Language {
    pub tree_sitter: tree_sitter_c2rust::Language,
    pub highlight_query: &'static str,
}

pub fn rust() -> Language {
    Language {
        tree_sitter: tree_sitter_rust::language(),
        highlight_query: tree_sitter_rust::HIGHLIGHT_QUERY,
    }
}
