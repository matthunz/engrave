use crate::layout::Layout;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::{use_signal, Signal};
use std::rc::Rc;
use tree_sitter_c2rust::Point;

mod line;
use line::Line;

mod use_editor;
pub use use_editor::{use_editor, UseEditor};

/// Text editor
#[component]
pub fn Editor(
    cx: Scope,
    editor: UseEditor,

    /// Font size of the editor text.
    #[props(default = 14.)]
    font_size: f64,
) -> Element {
    to_owned![editor, font_size];
    let line_height = editor.line_height;

    let lines_ref: Signal<Option<Rc<MountedData>>> = use_signal(cx, || None);

    let layout = use_signal(cx, || Layout::new(font_size, line_height));
    dioxus_signals::use_effect(cx, move || {
        layout.write().measure(editor.buffer().rope.lines())
    });

    let layout_ref = layout();
    let top_line = editor.list.start();
    let point = editor.cursor();
    let cursor_point = point
        .row
        .checked_sub(top_line)
        .map(|row| Point::new(row, point.column));
    let cursor_pos = cursor_point.map(|_| layout_ref.pos(point).unwrap_or_default());

    let line_values = use_signal(cx, Vec::new);
    dioxus_signals::use_effect(cx, move || {
        let layout_ref = layout();
        let top_line = layout_ref.line(editor.scroll() as _).unwrap_or_default();
        let bottom_line = top_line
            + (editor
                .container_size
                .read()
                .as_ref()
                .map(|rect| rect.height())
                .unwrap_or_default()
                / line_height)
                .floor() as usize
            + 1;

        let values_ref = editor.list.values.read();
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
            div { position: "absolute", top: "{top}px", right: 0, line_height: "inherit", "{line_idx + top_line + 1}" }
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
        Key::Enter => {
            let mut cursor_ref = editor.cursor_mut();
            editor
                .buffer_mut()
                .insert(cursor_ref.row, cursor_ref.column, "\n");
            cursor_ref.row += 1;
        }
        Key::ArrowUp => {
            let mut cursor_ref = editor.cursor_mut();
            cursor_ref.row = cursor_ref.row.saturating_sub(1);
        }
        Key::ArrowDown => editor.cursor_mut().row += 1,
        _ => {}
    };
    let onscroll = move |_| editor.list.scroll();

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
            prevent_default: "onkeydown",
            onmounted: move |event| editor.list.mounted.onmounted(event),
            onclick: move |_| async move {
                let mounted = editor.list.mounted.signal.read().clone();
                if let Some(mounted) = mounted {
                    mounted.set_focus(true).await.unwrap();
                    editor.focus();
                }
            },
            onfocusin: move |_| editor.focus(),
            onfocusout: move |_| editor.blur(),
            onkeydown: onkeydown,
            onscroll: onscroll,
            div { position: "relative", width: "50px", line_numbers.into_iter() }
            div {
                flex: 1,
                position: "relative",
                margin_left: "50px",
                height: "{height}px",
                onmounted: move |event| lines_ref.set(Some(event.data)),
                onmousedown: move |event| async move {
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
                        *editor.cursor_mut() = Point::new(line, col);
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

                lines.into_iter()
            }
        }
    )
}
