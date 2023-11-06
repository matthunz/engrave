use dioxus::prelude::*;
use engrave::{Editor, UseEditor};
use lookbook::{preview, Json, LookBook};

/// To-Do Task.
#[preview]
#[allow(non_snake_case)]
pub fn EditorPreview<'a>(
    cx: Scope<'a>,

    /// Font size
    #[lookbook(default = 18.)]
    font_size: Json<f64>,

    /// Line height
    #[lookbook(default = 24.)]
    line_height: Json<f64>,

    /// Font size
    #[lookbook(default = 400.)]
    height: Json<f64>,
) -> Element<'a> {
    let editor = UseEditor::builder()
        .font_size(font_size.0)
        .line_height(line_height.0)
        .height(height.0)
        .use_editor(cx, || include_str!("../examples/editor.rs"));

    render!(Editor { editor: editor })
}

fn app(cx: Scope) -> Element {
    render!(LookBook {
        home: |cx| render!("Home"),
        previews: [EditorPreview]
    })
}

fn main() {
    dioxus_web::launch(app)
}
