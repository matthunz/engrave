use dioxus::prelude::*;
use editor::Editor;
use log::LevelFilter;

fn main() {
    dioxus_logger::init(LevelFilter::Info).unwrap();
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    render!(Editor {})
}
