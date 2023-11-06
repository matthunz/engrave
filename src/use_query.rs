use crate::{buffer::RopeProvider, Buffer, Language};
use dioxus::prelude::{use_context, use_effect, Scope};
use dioxus_signals::{use_signal, Signal};
use tree_sitter_c2rust::{Query, QueryCursor, QueryMatches};

pub fn use_language<T>(cx: Scope<T>) -> Signal<Language> {
    *use_context(cx).unwrap()
}

pub fn use_query<T>(
    cx: Scope<T>,
    query: &str,
    buffer: Signal<Buffer>,
    mut onmatches: impl FnMut(QueryMatches<RopeProvider>) + 'static,
) {
    let query_signal = use_query_signal(cx, query);
    dioxus_signals::use_effect(cx, move || {
        let buffer_ref = buffer();
        let query = query_signal();

        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(
            &query,
            buffer_ref.tree.root_node(),
            RopeProvider {
                slice: buffer_ref.rope.slice(..),
            },
        );
        onmatches(matches)
    });
}

pub fn use_query_signal<T>(cx: Scope<T>, query: &str) -> Signal<Query> {
    let language = use_language(cx);
    let signal = use_signal(cx, || Query::new(language().tree_sitter, query).unwrap());

    use_effect(cx, &query.to_owned(), |q| async move {
        signal.set(Query::new(language().tree_sitter, &q).unwrap())
    });

    let query = query.to_owned();
    dioxus_signals::use_effect(cx, move || {
        signal.set(Query::new(language().tree_sitter, &query).unwrap())
    });

    signal
}
