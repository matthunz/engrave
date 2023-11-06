use dioxus::prelude::*;

#[component]
pub fn Cursor(cx: Scope, x: f64, y: f64, is_active: bool) -> Element {
    render!(div {
        position: "absolute",
        top: "{y}px",
        left: "{x}px",
        width: "3px",
        height: "24px",
        class: "cursor",
        z_index: 9,
        display: if *is_active { "block" } else { "none" }
    })
}
