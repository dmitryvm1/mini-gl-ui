# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`mini-gl-ui` is a minimal OpenGL UI library for Rust that provides a layered architecture for building 2D user interfaces. The library is structured in three distinct layers:

1. **Layer 1: OpenGL Primitives** (`src/primitives/`) - RAII wrappers around raw OpenGL objects (Shader, VertexBuffer, VertexArray, Texture)
2. **Layer 2: Renderer** (`src/renderer/`) - Higher-level rendering utilities built on primitives (QuadRenderer for 2D rectangles, text rendering via fontdue)
3. **Layer 3: UI Components** (`src/ui/`) - Pre-built widgets implementing the `Widget` trait (Button, Checkbox, TextBox, Label, Panel, Dropdown, Layouts)

## Commands

### Build and Test
```bash
# Build the library
cargo build

# Build with examples
cargo build --examples

# Run tests (no OpenGL context required - tests only verify logical behavior)
cargo test

# Run a specific test
cargo test test_button_creation
```

### Run Examples
```bash
# Run the main demo showcasing all UI components
cargo run --example demo

# Run the remote control demo
cargo run --example remote_demo
```

**Note**: Examples require X11 libraries on Linux:
- Ubuntu/Debian: `sudo apt-get install libx11-dev libxrandr-dev libxi-dev`
- Fedora: `sudo dnf install libX11-devel libXrandr-devel libXi-devel`

## Architecture Details

### Coordinate System
- Origin (0, 0) at **top-left corner**
- X-axis increases to the right
- Y-axis increases **downward**
- Uses orthographic projection for 2D UI: `Mat4::orthographic_rh_gl(0.0, width, height, 0.0, -1.0, 1.0)`

### Widget Trait
All UI components implement the `Widget` trait ([src/ui/mod.rs](src/ui/mod.rs)):
- `draw(&self, renderer: &QuadRenderer)` - Renders the widget
- `handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent>` - Processes input events and returns generated widget events
- `position()` / `size()` - Basic geometry
- `contains_point(point: Vec2) -> bool` - Hit detection

### Event System
Two parallel event flows:
1. **UiEvent** (input): `CursorMoved`, `MouseButton`, `Scroll`, `CharacterInput`, `KeyInput` - consumed by widgets via `handle_event()`
2. **WidgetEvent** (output): `ButtonClicked`, `CheckboxToggled`, `TextChanged`, `DropdownSelectionChanged`, `PanelDragStarted/Dragged/Ended`, `PanelToggleChanged` - emitted by widgets to notify application logic. Every variant now includes the widget's `id` for correlation.

### Remote Control Interface
The remote interface ([src/ui/remote.rs](src/ui/remote.rs)) provides runtime control of widgets via JSON commands over IPC/sockets:

- **RemoteCommandChannel**: Thread-safe queue for JSON commands. Use `spawn_reader_thread()` or `spawn_tcp_listener()` to feed commands from external sources.
- **RemoteUiSession**: Per-frame adapter that binds widget references to string IDs and applies queued commands.
- **RemoteUiHost**: Owning registry for fully remote-controlled UIs. Supports `create`/`destroy` methods and `attach_child` for composing layouts remotely.
- **RemoteCommand format**: `{ "id": "widget_id", "method": "set_position", "params": { "x": 10.0, "y": 20.0 } }`

Widgets expose setters (`set_position`, `set_size`, `set_text`, `set_label`, etc.) to enable remote mutation. Layouts expose `set_spacing`, `set_padding`, `set_cross_alignment`.

### Layout System
- **HorizontalLayout** / **VerticalLayout** ([src/ui/layout.rs](src/ui/layout.rs)): Automatic positioning of child widgets
- Configurable: `padding`, `spacing`, `cross_alignment` (Start, Center, End)
- Children are stored as `LayoutElement` enum variants (Button, Label, Checkbox, etc.)
- Call `add_child()` to append widgets; layout automatically recalculates positions

### Panel Container
**Panel** ([src/ui/panel.rs](src/ui/panel.rs)) is a draggable container with:
- Title bar for drag interaction
- Child widget support via `add_child(widget, offset)`
- Drag methods: `start_drag()`, `update_drag()`, `stop_drag()`
- Helper: `title_bar_contains_point()` for hit detection
- Children move with the panel when dragged

### Text Rendering
- Uses **fontdue** for text rasterization
- Text is rendered to OpenGL textures via `renderer::text::TextRenderer`
- Currently Label renders placeholder rectangles; check `TextRenderer` for actual text support

## Testing Strategy

Integration tests in [tests/ui_components.rs](tests/ui_components.rs) verify:
- Widget state management (pressed, checked, focused, dragging)
- Event handling (clicks, toggles, text input, panel dragging)
- Hit detection (`contains_point`)
- Remote command processing (RemoteUiSession, RemoteUiHost)
- Layout positioning and event forwarding
- Child widget interaction in panels and layouts

No OpenGL context is required for tests; they only verify logical behavior.

## Common Patterns

### Creating a Renderer
```rust
use mini_gl_ui::renderer::QuadRenderer;
use glam::Mat4;

let renderer = QuadRenderer::new().expect("Failed to create renderer");
let projection = Mat4::orthographic_rh_gl(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
renderer.set_projection(&projection);
```

### Creating and Using Widgets
```rust
use mini_gl_ui::{ui::*, Vec2, colors};

let mut button = Button::new(
    "example_button",
    Vec2::new(10.0, 10.0),
    Vec2::new(100.0, 40.0),
    "Click",
);
button.draw(&renderer);

// Handle events
if let Some(event) = button.handle_event(&UiEvent::MouseButton { ... }) {
    match event {
        WidgetEvent::ButtonClicked { id, label } => println!("Clicked {id}: {label}"),
        _ => {}
    }
}
```

### Using Layouts
```rust
let mut layout = VerticalLayout::new("example_layout", Vec2::new(10.0, 20.0))
    .with_spacing(8.0)
    .with_padding(Vec2::new(12.0, 12.0));

layout.add_child(Button::new(
    "layout_button",
    Vec2::ZERO,
    Vec2::new(120.0, 36.0),
    "Btn1",
));
layout.add_child(Label::new(
    "layout_label",
    Vec2::ZERO,
    Vec2::new(80.0, 20.0),
    "Text",
    colors::BLUE,
));

layout.draw(&renderer); // Draws all children at calculated positions
```

### Remote Control
```rust
use mini_gl_ui::ui::{RemoteCommandChannel, RemoteUiSession, RemoteCommand};
use serde_json::json;

let channel = RemoteCommandChannel::new();
channel.push(RemoteCommand {
    id: "my_button".to_string(),
    method: "set_label".to_string(),
    params: json!({ "text": "New Label" }),
});

let mut button = Button::new("my_button", ...);
let report = RemoteUiSession::new(&channel)
    .with_button("my_button", &mut button)
    .process();
```

## Code Style Notes

- OpenGL primitives use RAII pattern - resources are automatically cleaned up on drop
- Widgets are typically created with `new()` and configured with builder-style `with_*()` methods
- Event handling returns `Option<WidgetEvent>` - `None` means no event was generated
- The `Widget` trait provides default implementations for `handle_event()` and `contains_point()`
- Remote commands use serde_json `Value` for flexible parameter passing

## Dependencies

- **gl**: OpenGL bindings
- **glam**: Math library (Vec2, Vec3, Vec4, Mat4)
- **fontdue**: Text rasterization
- **serde/serde_json**: Serialization for remote commands
- **glfw** (dev): Window management for examples
