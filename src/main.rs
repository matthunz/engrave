use xilem::view::{button, h_stack, v_stack};
use xilem::{view::View, App, AppLauncher};

mod buffer;

pub mod language;

mod span;
pub use span::Span;

mod editor;
pub use editor::Editor;

fn app_logic(data: &mut i32) -> impl View<i32> {
    Editor::new(include_str!("../example.rs"))
}

fn main() {
    let app = App::new(0, app_logic);
    AppLauncher::new(app).run()
}
