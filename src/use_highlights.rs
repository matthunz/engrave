use crate::{use_language, use_query, Buffer};
use dioxus::prelude::Scope;
use dioxus_signals::{use_signal, Signal};
use tree_sitter_c2rust::Range;

/// Highlighted text span.
#[derive(Debug)]
pub struct Highlight {
    pub kind: String,
    pub range: Range,
}

pub fn use_highlights<T>(cx: Scope<T>, buffer: Signal<Buffer>) -> Signal<Vec<Highlight>> {
    let language = use_language(cx);
    let highlights = use_signal(cx, || Vec::new());
    use_query(cx, language().highlight_query, buffer, move |matches| {
        let items = matches
            .flat_map(|mat| {
                mat.captures.iter().map(|capture| {
                    let range = capture.node.range();
                    let kind = capture.node.kind().to_owned();
                    Highlight { range, kind }
                })
            })
            .collect();
        highlights.set(items);
    });
    highlights
}
