# Engrave
Text editor for desktop and the web with Rust and Dioxus.

```rust
use dioxus::prelude::*;
use editor::{language, use_buffer, Editor};

fn app(cx: Scope) -> Element {
    let editor = use_editor(cx, language::rust(), || include_str!("editor.rs"));

    render!(Editor { editor: editor })
}
```
