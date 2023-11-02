use crate::{layout::Layout, Buffer, Span};
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::use_signal;
use tree_sitter_c2rust::Point;

#[component]
pub fn Editor(cx: Scope) -> Element {
    let editor = use_signal(cx, || Buffer::new(include_str!("../example.rs")));
    let container_ref = use_signal(cx, || None);

    let cursor = use_signal(cx, || Point::new(0, 0));
    let is_focused = use_signal(cx, || false);

    let layout = Layout::new(editor().rope.lines());
    let mut y = 0.;
    let lines = editor()
        .lines()
        .into_iter()
        .zip(layout.lines())
        .enumerate()
        .map(|(line_idx, (spans, line))| {
            let top = y;
            y += line.height;

            render!(Line {
                key: "{line_idx}",
                spans: spans,
                top: top,
                height: line.height,
                is_selected: *is_focused() && line_idx == cursor().row
            })
        });

    render!(div {
        position: "relative",
        width: "800px",
        height: "600px",
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
fn Line(cx: Scope, spans: Vec<Span>, is_selected: bool, top: f64, height: f64) -> Element {
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
        position: "absolute",
        top: "{top}px",
        width: "100%",
        height: "{height}px",
        white_space: "pre",
        border_top: border,
        border_bottom: border,
        spans
    })
}

#[component]
fn LineSpan(cx: Scope, span: Span) -> Element {
    let color = match span.kind.as_deref() {
        Some(s) => match &**s {
            "fn" | "struct" | "pub" | "use" | "let" | "match" => "rgb(207, 34, 46)",
            "" => "#427b58",
            "attribute_item" | "type_identifier" | "identifier" => "rgb(96, 59, 179)",
            "primitive_type" | "boolean_identifier" | "::" | "*" => "rgb(5, 80, 174)",
            "{" | "}" => "#076678",
            "string_literal" => "#21262d",
            ";" => "#ccc",
            _ => "#000",
        },
        _ => "#000",
    };

    render!(
        span { color: color, "data-kind": "{span.kind.clone().unwrap_or_default()}", "{span.text}" }
    )
}
