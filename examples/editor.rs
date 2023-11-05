use dioxus::prelude::*;
use engrave::{language, Editor, UseEditor};
use log::LevelFilter;

fn app(cx: Scope) -> Element {
    let editor = UseEditor::builder()
        .language(language::rust())
        .height(600.)
        .use_editor(cx, || include_str!("../src/editor/use_editor.rs"));

    render!(Editor { editor: editor })
}

fn main() {
    dioxus_logger::init(LevelFilter::Info).unwrap();
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}
