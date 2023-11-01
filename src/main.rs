use dioxus::prelude::*;
use dioxus_signals::use_signal;
use editor::Editor;
use log::LevelFilter;

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let editor = use_signal(cx, || Editor::new("fn main() {}"));
    let spans = editor().spans().into_iter().map(|span| {
        let color = match span.kind.as_deref() {
            Some("fn") => "red",
            _ => "#000"
        };
        render!(span {
            color: color,
            span.text
        })
    });

    render!(div {font: "14px monospace", spans })
}
