use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::use_signal;
use editor::{Editor, Span};
use log::LevelFilter;
use tree_sitter_c2rust::Point;

fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let editor = use_signal(cx, || Editor::new(include_str!("../example.rs")));
    let cursor = use_signal(cx, || Point::new(0, 0));

    let lines = editor()
        .lines()
        .into_iter()
        .enumerate()
        .map(|(line_idx, spans)| {
            render!(Line {
                key: "{line_idx}",
                spans: spans
            })
        });

    render!(div {
        font: "18px monospace",
        line_height: "24px",
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
        lines
    })
}

#[component]
fn Line(cx: Scope, spans: Vec<Span>) -> Element {
    let spans = spans.into_iter().enumerate().map(|(span_idx, span)| {
        render!(LineSpan {
            key: "{span_idx}",
            span: span.clone()
        })
    });

    render!(div {
        white_space: "pre",
        spans
    })
}

#[component]
fn LineSpan(cx: Scope, span: Span) -> Element {
    let color = match span.kind.as_deref() {
        Some(s) => match s {
            "fn" | "struct" | "pub" | "let" | "match" => "rgb(207, 34, 46)",
            "identifier" => "#427b58",
            "attribute_item" | "type_identifier" => "rgb(96, 59, 179)",
            "primitive_type" => "rgb(5, 80, 174)",
            "{" | "}" => "#076678",
            _ => "#000",
        },
        _ => "#000",
    };

    render!(
        span { color: color, "data-kind": "{span.kind.clone().unwrap_or_default()}", "{span.text}" }
    )
}
