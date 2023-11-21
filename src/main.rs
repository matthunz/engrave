use vello::peniko::Color;

use xilem::{view::View, App, AppLauncher};

mod buffer;

pub mod language;

mod span;
pub use span::Span;

mod editor;
pub use editor::Editor;

pub fn color(kind: Option<&str>) -> Color {
    match kind {
        Some(ref s) => match &**s {
            "fn" | "struct" | "pub" | "use" | "let" | "match" | "async" | "unsafe" | "move"
            | "|" | "impl" | "mutable_specifier" | "self" => {
                Color::rgb(193. / 255., 128. / 255., 138. / 255.)
            }
            "primitive_type" | "boolean_identifier" | "::" | "*" | "attribute_item"
            | "identifier" | "type_identifier" | "!" | "'" => {
                Color::rgb(96. / 255., 59. / 255., 179. / 255.)
            }
            "string_literal" | "integer_literal" => {
                Color::rgb(208. / 255., 148. / 255., 208. / 255.)
            }
            ";" | "," | "<" | ">" | ":" | "(" | ")" | "{" | "}" | "=>" | "&" => {
                Color::rgb(209. / 255., 209. / 255., 209. / 255.)
            }
            _ => Color::WHITE,
        },
        _ => Color::WHITE,
    }
}

fn app_logic(_data: &mut i32) -> impl View<i32> {
    Editor::new(include_str!("main.rs"))
}

fn main() {
    let app = App::new(0, app_logic);
    AppLauncher::new(app).run()
}
