# Editor
Text editor for desktop and the web with Dioxus

```rust
use dioxus::prelude::*;
use editor::{use_buffer, Editor};

fn app(cx: Scope) -> Element {
    let buffer = use_buffer(cx, || include_str!("editor.rs"));

    render!(Editor { buffer: buffer })
}
```