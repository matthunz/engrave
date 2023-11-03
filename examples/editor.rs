use dioxus::prelude::*;
use engrave::{use_editor, Editor};
use log::LevelFilter;

fn app(cx: Scope) -> Element {
    let editor = use_editor(cx, tree_sitter_rust::language(), || {
        include_str!("editor.rs")
    });

    render!(Editor { editor: editor })
}

fn main() {
    dioxus_logger::init(LevelFilter::Info).unwrap();
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}
