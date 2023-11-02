use dioxus::prelude::*;
use editor::{use_buffer, Editor};
use log::LevelFilter;

fn app(cx: Scope) -> Element {
    let buffer = use_buffer(cx, || include_str!("editor.rs"));

    render!( Editor { buffer: buffer } )
}

fn main() {
    dioxus_logger::init(LevelFilter::Info).unwrap();
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}
