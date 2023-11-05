use crate::layout::Layout;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::{use_signal, Signal};
use std::rc::Rc;
use tree_sitter_c2rust::Point;

mod line;
use line::Line;

mod use_editor;
pub use use_editor::{Builder, UseEditor};

/// Text editor
#[component]
pub fn Editor<'a>(
    cx: Scope<'a>,
    editor: &'a UseEditor,

    /// Font size of the editor text.
    #[props(default = 14.)]
    font_size: f64,
) -> Element<'a> {
    to_owned![editor, font_size];
    let line_height = editor.line_height;

    let lines_ref: Signal<Option<Rc<MountedData>>> = use_signal(cx, || None);

    let layout = use_signal(cx, || Layout::new(font_size, line_height));
    let buffer = editor.buffer;
    dioxus_signals::use_effect(cx, move || layout.write().measure(buffer().rope.lines()));

    let layout_ref = layout();
    let top_line = editor.list.scroll_range.start();
    let point = editor.cursor();
    let cursor_point = point
        .row
        .checked_sub(top_line)
        .map(|row| Point::new(row, point.column));
    let cursor_pos = cursor_point.map(|_| layout_ref.pos(point).unwrap_or_default());

    let anchor: Signal<Option<Point>> = use_signal(cx, || None);
    let is_mouse_down = use_signal(cx, || false);

    let line_values = use_signal(cx, Vec::new);
    let container_size = editor.container_size;
    let scroll = editor.list.scroll_range.scroll;
    let values = editor.list.lazy.values;
    dioxus_signals::use_effect(cx, move || {
        let layout_ref = layout();
        let top_line = layout_ref.line(*scroll() as _).unwrap_or_default();
        let _bottom_line = top_line
            + (container_size()
                .as_ref()
                .map(|rect| rect.height())
                .unwrap_or_default()
                / line_height)
                .floor() as usize
            + 1;

        let values_ref = values();
        let values = values_ref
            .iter()
            .cloned()
            .zip(layout_ref.lines().iter().cloned())
            .enumerate()
            .collect();
        line_values.set(values)
    });

    let mut line_numbers = Vec::new();
    let mut lines = Vec::new();

    let mut y = top_line as f64 * line_height;
    for (line_idx, (spans, line)) in line_values {
        let top = y;
        y += line.height;

        let line_number = render!(
            div { position: "absolute", top: "{top}px", right: 0, color: "#888", line_height: "inherit",
                "{line_idx + top_line + 1}"
            }
        );
        line_numbers.push(line_number);

        let is_selected = if let Some(point) = cursor_point {
            editor.is_focused() && line_idx == point.row
        } else {
            false
        };

        let line = render!(Line {
            key: "{line_idx}",
            spans: spans,
            top: top,
            height: line.height,
            is_selected: is_selected
        });
        lines.push(line);
    }

    let height = editor.buffer().rope.len_lines() as f64 * line_height;
    let onkeydown = move |event: KeyboardEvent| match event.key() {
        Key::Character(text) => editor.insert(&text),
        Key::Enter => editor.insert("\n"),
        Key::ArrowUp => {
            let mut cursor_ref = editor.cursor_mut();
            cursor_ref.row = cursor_ref.row.saturating_sub(1);
        }
        Key::ArrowDown => editor.cursor_mut().row += 1,
        _ => {}
    };
    let onscroll = move |_| editor.list.scroll();

    let onmousemove = move |event: MouseEvent| async move {
        if *is_mouse_down() {
            let mut anchor_ref = anchor.write();
            if let Some(_point) = &mut *anchor_ref {
                let lines_elem = lines_ref.unwrap();
                let bounds = lines_elem.get_client_rect().await.unwrap();
                if let Some((line, col_cell)) = layout().target(
                    event.client_coordinates().x - bounds.origin.x,
                    event.client_coordinates().y - bounds.origin.y,
                ) {
                    let col = col_cell.unwrap_or_default();
                    *anchor_ref = Some(Point::new(line, col));
                }
            }
        }
    };

    let selection = &anchor().and_then(|anchor| {
        if let Some(_anchor_pos) = layout().pos(anchor) {
            if let Some(_cursor_pos) = cursor_pos {
                let cursor = cursor_point.unwrap();

                let start_line = anchor.row.min(cursor.row);
                let end_line = anchor.row.max(cursor.row);

                let start_col = anchor.column.min(cursor.column);
                let end_col = anchor.column.max(cursor.column);

                let top = start_line.saturating_sub(editor.list.scroll_range.start()) as f64
                    * line_height;

                let lines = (0..end_line - start_line).map(|idx| {
                    let line_top = top + idx as f64 * line_height;
                    render!(div {
                        position: "absolute",
                        top: "{line_top}px",
                        left: 0,
                        width: "100%",
                        height: "{line_height}px",
                        background: "rgb(181, 215, 251)"
                    })
                });

                let start_pos = layout_ref
                    .pos(Point::new(end_line, start_col))
                    .unwrap_or_default();
                let end_pos = layout_ref
                    .pos(Point::new(end_line, end_col))
                    .unwrap_or_default();
                let width = end_pos[0] - start_pos[0];

                render! {
                    lines,
                    div {
                        position: "absolute",
                        top: "{top + ((end_line - start_line) as f64 * line_height)}px",
                        left: "{start_pos[0]}px",
                        width: "{width}px",
                        height: "{line_height}px",
                        background: "rgb(181, 215, 251)"
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    });

    let mounted = editor.list.mounted;
    let cursor = editor.cursor;

    render!(
        div {
            position: "relative",
            display: "flex",
            flex_direction: "row",
            width: "800px",
            height: "{editor.height}px",
            margin: "50px auto",
            font: "{font_size}px monospace",
            line_height: "26px",
            border: "2px solid #ccc",
            overflow: "auto",
            tabindex: 0,
            outline: "none",
            user_select: "none",
            webkit_user_select: "none",
            prevent_default: "onkeydown",
            onmounted: move |event| editor.list.mounted.onmounted(event),
            onclick: move |_| {
                editor.focus();
                async move {
                    let mounted = mounted.signal.read().clone();
                    if let Some(mounted) = mounted {
                        mounted.set_focus(true).await.unwrap();
                    }
                }
            },
            onfocusin: move |_| editor.focus(),
            onfocusout: move |_| editor.blur(),
            onkeydown: onkeydown,
            onscroll: onscroll,
            onmousemove: onmousemove,
            onmouseup: move |_| {
                is_mouse_down.set(false);
                if let Some(cursor_point) = cursor_point {
                    let mut anchor_ref = anchor.write();
                    if let Some(anchor_point) = &*anchor_ref {
                        if cursor_point == *anchor_point {
                            *anchor_ref = None;
                        }
                    }
                }
            },
            div { position: "relative", width: "50px", line_numbers.into_iter() }
            div {
                flex: 1,
                position: "relative",
                margin_left: "50px",
                height: "{height}px",
                onmounted: move |event| lines_ref.set(Some(event.data)),
                onmousedown: move |event| async move {
                    is_mouse_down.set(true);
                    let lines_elem = lines_ref.unwrap();
                    let bounds = lines_elem.get_client_rect().await.unwrap();
                    if let Some((line, col_cell))
                        = layout()
                            .target(
                                event.client_coordinates().x - bounds.origin.x,
                                event.client_coordinates().y - bounds.origin.y,
                            )
                    {
                        let col = col_cell.unwrap_or_default();
                        *cursor.write() = Point::new(line, col);
                        anchor.set(Some(Point::new(line, col)));
                    }
                },
                if let Some(cursor_pos) = cursor_pos {
                    render!(div {
                        position: "absolute",
                        top: "{cursor_pos[1]}px",
                        left: "{cursor_pos[0]}px",
                        width: "3px",
                        height: "24px",
                        class: "cursor",
                        z_index: 9,
                        display: if editor.is_focused() { "block" } else { "none" }
                    })
                }
                selection,
                lines.into_iter()
            }
        }
    )
}
