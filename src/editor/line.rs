use crate::Span;
use dioxus::prelude::*;

#[component]
pub fn Line(cx: Scope, spans: Vec<Span>, is_selected: bool, top: f64, height: f64) -> Element {
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
