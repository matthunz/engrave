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
    let container_ref = use_signal(cx, || None);

    let cursor = use_signal(cx, || Point::new(0, 0));
    let is_focused = use_signal(cx, || false);

    let lines = editor()
        .lines()
        .into_iter()
        .enumerate()
        .map(|(line_idx, spans)| {
            render!(Line {
                key: "{line_idx}",
                spans: spans,
                is_selected: *is_focused() && line_idx == cursor().row
            })
        });

    render!(div {
        width: "800px",
        margin: "50px auto",
        font: "18px monospace",
        line_height: "24px",
        border: "2px solid #ccc",
        overflow: "auto",
        tabindex: 0,
        outline: "none",
        prevent_default: "onkeydown",
        onmounted: move |event| container_ref.set(Some(event.data)),
        onclick: move |_| async move {
            if let Some(mounted) = &*container_ref() {
                mounted.set_focus(true).await.unwrap();
                is_focused.set(true);
            }
        },
        onfocusin: move |_| is_focused.set(true),
        onfocusout: move |_| is_focused.set(false),
        onkeydown: move |event| {
            match event.key() {
                Key::Character(text) => {
                    let mut cursor_ref = cursor.write();
                    editor
                        .write()
                        .insert(cursor_ref.row, cursor_ref.column, &text);
                    cursor_ref.column += text.len();
                }
                Key::Enter => {
                    let mut cursor_ref = cursor.write();
                    editor
                        .write()
                        .insert(cursor_ref.row, cursor_ref.column, "\n");
                    cursor_ref.row += 1;
                }
                Key::ArrowUp => {
                    let mut cursor_ref = cursor.write();
                    cursor_ref.row = cursor_ref.row.saturating_sub(1);
                }
                Key::ArrowDown => {
                    cursor.write().row += 1;
                }
                _ => {}
            }
        },
        lines
    })
}

#[component]
fn Line(cx: Scope, spans: Vec<Span>, is_selected: bool) -> Element {
    let spans = spans.into_iter().enumerate().map(|(span_idx, span)| {
        render!(LineSpan {
            key: "{span_idx}",
            span: span.clone()
        })
    });
    let border = if *is_selected {
        "2px solid #ccc"
    } else {
        "2px solid rgba(0, 0, 0, 0)"
    };

    render!(div {
        white_space: "pre",
        border_top: border,
        border_bottom: border,
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
