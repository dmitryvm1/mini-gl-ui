# mini-gl-ui

A minimal OpenGL UI engine for drawing user interfaces in Rust.

## Features

This library provides simple and easy-to-use primitives for building OpenGL-based user interfaces:

### OpenGL Primitives
- **Shader**: Wrapper around OpenGL shader programs with utility methods for setting uniforms
- **VertexBuffer**: Wrapper around OpenGL Vertex Buffer Objects (VBO)
- **VertexArray**: Wrapper around OpenGL Vertex Array Objects (VAO)
- **Texture**: Wrapper around OpenGL textures

### Renderer
- **QuadRenderer**: Efficient renderer for drawing 2D rectangles with solid colors and outlines

### UI Components
- **Label**: Display text (currently renders as colored rectangles)
- **Button**: Clickable button with hover and pressed states
- **Checkbox**: Toggle checkbox with checked/unchecked states
- **TextBox**: Text input field with focus states
- **Panel**: Draggable panel with title bar and +/- collapse toggle

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
mini_gl_ui = "0.1.0"
gl = "0.14"
glfw = "0.54"  # Or your preferred windowing library
glam = "0.24"
```

### Basic Example

```rust
use mini_gl_ui::{colors, renderer::QuadRenderer, ui::*, Vec2};
use glam::Mat4;

// Initialize OpenGL context (using glfw, glutin, sdl2, etc.)
// ...

// Create renderer
let renderer = QuadRenderer::new().expect("Failed to create renderer");

// Set up orthographic projection
let projection = Mat4::orthographic_rh_gl(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
renderer.set_projection(&projection);

// Create UI components
let button = Button::new(
    Vec2::new(50.0, 50.0),
    Vec2::new(150.0, 40.0),
    "Click Me".to_string(),
);

let checkbox = Checkbox::new(
    Vec2::new(50.0, 110.0),
    Vec2::new(30.0, 30.0),
    "Option".to_string(),
);

let mut panel = Panel::new(
    Vec2::new(250.0, 50.0),
    Vec2::new(400.0, 300.0),
    "My Panel".to_string(),
);

// In your render loop:
button.draw(&renderer);
checkbox.draw(&renderer);
panel.draw(&renderer);
```

Panels display a `+` icon when collapsed and a `-` icon when expanded, letting users quickly hide or reveal the content area.

### Running the Demo

The demo example showcases all UI components:

```bash
cargo run --example demo
```

**Note**: The demo requires X11 libraries on Linux. You may need to install them:
```bash
# Ubuntu/Debian
sudo apt-get install libx11-dev libxrandr-dev libxi-dev

# Fedora
sudo dnf install libX11-devel libXrandr-devel libXi-devel
```

## Architecture

### Layer 1: OpenGL Primitives (`primitives` module)
Wraps low-level OpenGL API calls into safe, easy-to-use Rust structs:
- Manages OpenGL object lifecycle with RAII
- Provides simple APIs for common operations
- Handles error checking and resource cleanup

### Layer 2: Renderer (`renderer` module)
Built on top of primitives to provide higher-level rendering utilities:
- QuadRenderer for efficient 2D rectangle rendering
- Handles projection matrices and coordinate transformations

### Layer 3: UI Components (`ui` module)
Pre-built UI widgets ready to use:
- Each widget implements the `Widget` trait
- Handles state management (hover, pressed, focus, etc.)
- Supports user interaction (clicking, dragging, text input)

## Building

Build the library:
```bash
cargo build
```

Build with examples:
```bash
cargo build --examples
```

Run tests:
```bash
cargo test
```

## License

This project is open source and available under the MIT License.
