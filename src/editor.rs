use crate::{layout::Layout, Buffer, Span};
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::{use_signal, Signal};
use gloo_timers::future::TimeoutFuture;
use std::rc::Rc;
use tree_sitter_c2rust::Point;

#[component]
pub fn Editor(cx: Scope) -> Element {
    let editor = use_signal(cx, || Buffer::new(include_str!("../example.rs")));
    let container_ref: Signal<Option<Rc<MountedData>>> = use_signal(cx, || None);

    let cursor = use_signal(cx, || Point::new(0, 0));
    let is_focused = use_signal(cx, || false);

    let layout = use_signal(cx, || Layout::new(editor().rope.lines()));
    let layout_ref = layout();

    let mut y = 0.;
    let lines_and_numbers: Vec<_> = editor()
        .lines()
        .into_iter()
        .zip(layout_ref.lines())
        .enumerate()
        .map(|(line_idx, (spans, line))| {
            let top = y;
            y += line.height;

            let line_number = render!(div { position: "absolute", top: "{top}px", right: 0, line_height: "inherit", "{line_idx}" });

            let line = render!(
                Line {
                    key: "{line_idx}",
                    spans: spans,
                    top: top,
                    height: line.height,
                    is_selected: *is_focused() && line_idx == cursor().row
                }
            );

            (line_number, line)
        })
        .collect();

    let lines = lines_and_numbers.clone().into_iter().map(|(_, line)| line);
    let line_numbers = lines_and_numbers.into_iter().map(|(n, _)| n);

    let cursor_pos = layout_ref.pos(cursor().clone());
    let is_cursor_shown = use_signal(cx, || true);
    use_effect(cx, (), move |_| async move {
        loop {
            TimeoutFuture::new(500).await;
            is_cursor_shown.toggle();
        }
    });

    log::info!("{:?}", cursor());

    render!(
        div {
            position: "relative",
            display: "flex",
            flex_direction: "row",
            width: "800px",
            height: "600px",
            margin: "50px auto",
            font: "16px monospace",
            line_height: "26px",
            border: "2px solid #ccc",
            overflow: "auto",
            tabindex: 0,
            outline: "none",
            prevent_default: "onkeydown",
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
                        editor.write().insert(cursor_ref.row, cursor_ref.column, &text);
                        cursor_ref.column += text.len();
                    }
                    Key::Enter => {
                        let mut cursor_ref = cursor.write();
                        editor.write().insert(cursor_ref.row, cursor_ref.column, "\n");
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
            div { position: "relative", width: "50px", line_numbers }
            div {
                flex: 1,
                position: "relative",
                margin_left: "50px",
                onmounted: move |event| container_ref.set(Some(event.data)),
                onmousedown: move |event| async move {
                    let bounds = container_ref().as_ref().unwrap().get_client_rect().await.unwrap();
                    if let Some((line, col_cell))
                        = layout()
                            .target(
                                event.client_coordinates().x - bounds.origin.x,
                                event.client_coordinates().y - bounds.origin.y,
                            )
                    {
                        if let Some(col) = col_cell {
                            cursor.set(Point::new(line, col));
                        } else {
                            cursor.set(Point::new(line, 0));
                        }
                    }
                },
                div {
                    position: "absolute",
                    top: "{cursor_pos[1]}px",
                    left: "{cursor_pos[0]}px",
                    width: "3px",
                    height: "24px",
                    background: if *is_cursor_shown() { "#000" } else { "none" },
                    z_index: 9
                }
                lines
            }
        }
    )
}

#[component]
fn Line(cx: Scope, spans: Vec<Span>, is_selected: bool, top: f64, height: f64) -> Element {
    let spans = spans.into_iter().enumerate().map(|(span_idx, span)| {
        render!(LineSpan {
            key: "{span_idx}",
            span: span.clone()
        })
    });

    render!(div {
        position: "absolute",
        top: "{top}px",
        width: "100%",
        height: "{height}px",
        white_space: "pre",
        border: if *is_selected {
            "2px solid #c6cdd5"
        } else {
            "2px solid rgba(0, 0, 0, 0)"
        },
        box_sizing: "border-box",
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
