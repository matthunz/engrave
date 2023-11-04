use dioxus::prelude::*;
use engrave::{language, use_editor, Editor};
use log::LevelFilter;

fn app(cx: Scope) -> Element {
    let editor = use_editor(
        cx,
        language::rust(),
        || include_str!("../src/editor/use_editor.rs"),
        400.,
        24.,
    );

    render!(Editor { editor: editor })
}

fn main() {
    dioxus_logger::init(LevelFilter::Info).unwrap();
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}
