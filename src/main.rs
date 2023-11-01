use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::use_signal;
use editor::Editor;
use log::LevelFilter;
use tree_sitter_c2rust::Point;

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let editor = use_signal(cx, || Editor::new("fn main() {}"));
    let cursor = use_signal(cx, || Point::new(0, 0));

    let spans = editor().spans().into_iter().map(|span| {
        let color = match span.kind.as_deref() {
            Some("fn") => "red",
            _ => "#000",
        };
        render!(
            span { color: color, span.text }
        )
    });

    render!(div {
        font: "14px monospace",
        tabindex: 0,
        onkeydown: move |event| {
            match event.key() {
                Key::Character(text) => {
                    let mut cursor_ref = cursor.write();
                    editor
                        .write()
                        .insert(cursor_ref.row, cursor_ref.column, &text);
                    cursor_ref.column += text.len();
                }
                _ => {}
            }
        },
        spans
    })
}
