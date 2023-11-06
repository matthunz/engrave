use crate::{editor::cursor::Cursor, layout::Layout, Range};
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_signals::{use_signal, Signal};
use std::rc::Rc;
use tree_sitter_c2rust::Point;

mod cursor;

mod line;
use line::Line;

mod use_editor;
pub use use_editor::{Builder, UseEditor};

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
    let buffer = editor.buffer;
    dioxus_signals::use_effect(cx, move || layout.write().measure(buffer().rope.lines()));

    let layout_ref = layout();
    let top_line = editor.list.scroll_range.start();

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

        let n = line_idx + top_line;

        let is_selected = if let Some(selection) = editor.selections.read().first() {
            editor.is_focused() && n == selection.start.row
        } else {
            false
        };

        let line_number = render!(
            div { position: "absolute", top: "{top}px", right: 0, color: if is_selected { "#000" } else { "#888" }, line_height: "inherit", "{n + 1}" }
        );
        line_numbers.push(line_number);

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

        _ => {}
    };
    let onscroll = move |_| editor.list.scroll();

    let editor_clone = editor.clone();
    let onmousemove = move |event: MouseEvent| async move {
        if *is_mouse_down() {
            if let Some(selection) = editor_clone.selections.write().last_mut() {
                let lines_elem = lines_ref.unwrap();
                let bounds = lines_elem.get_client_rect().await.unwrap();
                if let Some((line, col_cell)) = layout().target(
                    event.client_coordinates().x - bounds.origin.x,
                    event.client_coordinates().y - bounds.origin.y,
                ) {
                    let col = col_cell.unwrap_or_default();
                    selection.end = Point::new(line, col);
                }
            }
        }
    };

    let mounted = editor.list.mounted;
    let editor_ref = editor.selections.read();
    let cursors = editor_ref.iter().map(|selection| {
        let [x, y] = layout_ref.pos(selection.start).unwrap_or_default();

        render!(Cursor {
            x: x,
            y: y,
            is_active: editor.is_focused()
        })
    });

    let editor_clone = editor.clone();
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
            onmouseup: move |_| is_mouse_down.set(false),
            div { position: "relative", width: "50px", line_numbers.into_iter() }
            div {
                flex: 1,
                position: "relative",
                margin_left: "50px",
                height: "{height}px",
                cursor: "text",
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
                        let mut selections = editor_clone.selections.write();
                        selections.clear();
                        selections.push(Range::new(Point::new(line, col), Point::new(line, col)));
                    }
                },
                cursors,
                lines.into_iter()
            }
        }
    )
}
