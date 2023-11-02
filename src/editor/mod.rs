use crate::{layout::Layout, Buffer};
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::{use_signal, Signal};
use std::rc::Rc;
use tree_sitter_c2rust::Point;

mod line;
use line::Line;

#[component]
pub fn Editor(cx: Scope, buffer: Signal<Buffer>) -> Element {
    let container_ref: Signal<Option<Rc<MountedData>>> = use_signal(cx, || None);
    let layout = use_signal(cx, || Layout::new());

    to_owned![buffer];
    dioxus_signals::use_effect(cx, move || layout.write().measure(buffer().rope.lines()));

    let cursor = use_signal(cx, || Point::new(0, 0));
    let is_focused = use_signal(cx, || false);

    let layout_ref = layout();
    let mut y = 0.;
    let mut line_numbers = Vec::new();
    let mut lines = Vec::new();
    for (line_idx, (spans, line)) in buffer()
        .lines()
        .into_iter()
        .zip(layout_ref.lines())
        .enumerate()
    {
        let top = y;
        y += line.height;

        let line_number = render!(div { position: "absolute", top: "{top}px", right: 0, line_height: "inherit", "{line_idx + 1}" });
        line_numbers.push(line_number);

        let line = render!(Line {
            key: "{line_idx}",
            spans: spans,
            top: top,
            height: line.height,
            is_selected: *is_focused() && line_idx == cursor().row
        });
        lines.push(line);
    }

    let cursor_pos = layout_ref.pos(cursor().clone());
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
                        buffer.write().insert(cursor_ref.row, cursor_ref.column, &text);
                        cursor_ref.column += text.len();
                    }
                    Key::Enter => {
                        let mut cursor_ref = cursor.write();
                        buffer.write().insert(cursor_ref.row, cursor_ref.column, "\n");
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
            div { position: "relative", width: "50px", line_numbers.into_iter() }
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
                    class: "cursor",
                    z_index: 9,
                    display: if *is_focused() { "block" } else { "none" }
                }
                lines.into_iter()
            }
        }
    )
}
