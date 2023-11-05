use crate::Span;
use dioxus::prelude::*;

#[component]
pub fn Line(cx: Scope, spans: Vec<Span>, is_selected: bool, top: f64, height: f64) -> Element {
    let spans = spans.iter().enumerate().map(|(span_idx, span)| {
        render!( LineSpan { key: "{span_idx}", span: span.clone() } )
    });

    render!(
        div {
            position: "absolute",
            top: "{top}px",
            width: "100%",
            height: "{height}px",
            white_space: "pre",
            border: if *is_selected { "2px solid #c6cdd5" } else { "2px solid rgba(0, 0, 0, 0)" },
            box_sizing: "border-box",
            spans
        }
    )
}

#[component]
fn LineSpan(cx: Scope, span: Span) -> Element {
    let color = match span.kind {
        Some(ref s) => match &**s {
            "fn" | "struct" | "pub" | "use" | "let" | "match" | "async" | "unsafe" | "move"
            | "|" | "impl" => "rgb(207, 34, 46)",
            "attribute_item" | "identifier" | "type_identifier" | "!" | "'" => "rgb(96, 59, 179)",
            "primitive_type" | "boolean_identifier" | "::" | "*" => "rgb(5, 80, 174)",
            "string_literal" | "integer_literal" => "rgb(7, 69, 124)",
            "{" | "}" => "#076678",
            "(" | ")" | "=>" | "&" => "#faa356",
            ";" | "," | "<" | ">" | ":" => "#ccc",
            _ => "#000",
        },
        _ => "#000",
    };

    render!(
        span { color: color, "data-kind": "{span.kind.as_deref().unwrap_or(\"\")}", "{span.text}" }
    )
}
