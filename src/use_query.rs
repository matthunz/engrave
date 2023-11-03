use dioxus::prelude::{use_context, use_effect, Scope};
use dioxus_signals::{use_signal, Signal};
use tree_sitter_c2rust::{ Query};
use crate::Language;

pub fn use_language<T>(cx: Scope<T>) -> Signal<Language> {
    *use_context(cx).unwrap()
}

pub fn use_query<T>(cx: Scope<T>, query: &str) -> Signal<Query> {
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
