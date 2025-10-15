# Architecture

This document describes the architecture of the mini-gl-ui library.

## Overview

The mini-gl-ui library is designed in three distinct layers, each building on top of the previous one:

```
┌─────────────────────────────────────┐
│     UI Components (Layer 3)        │
│  Button, Checkbox, Label, etc.     │
├─────────────────────────────────────┤
│     Renderer (Layer 2)              │
│  QuadRenderer, Text Rendering       │
├─────────────────────────────────────┤
│     OpenGL Primitives (Layer 1)    │
│  Shader, VBO, VAO, Texture          │
└─────────────────────────────────────┘
```

## Layer 1: OpenGL Primitives (`src/primitives/`)

This layer wraps low-level OpenGL API calls into safe, easy-to-use Rust structs.

### Components

#### `Shader` (`shader.rs`)
- Compiles vertex and fragment shaders
- Links them into a shader program
- Provides methods to set uniform variables (mat4, vec4, vec2, float, int)
- Automatic cleanup on drop

#### `VertexBuffer` (`buffer.rs`)
- Wraps OpenGL VBO (Vertex Buffer Object)
- Handles buffer binding and data upload
- RAII pattern for automatic resource cleanup

#### `VertexArray` (`buffer.rs`)
- Wraps OpenGL VAO (Vertex Array Object)
- Configures vertex attributes
- Manages vertex attribute state

#### `Texture` (`texture.rs`)
- Wraps OpenGL textures
- Supports texture creation from data
- Handles texture binding to texture units

### Design Principles

1. **RAII**: All OpenGL resources are automatically cleaned up when dropped
2. **Type Safety**: Uses Rust's type system to prevent common OpenGL errors
3. **Simplicity**: Provides simple APIs hiding complex OpenGL state management

## Layer 2: Renderer (`src/renderer/`)

This layer builds on primitives to provide higher-level rendering utilities.

### Components

#### `QuadRenderer` (`quad.rs`)
- Efficient renderer for 2D rectangles
- Uses a single shader and VAO/VBO setup
- Methods:
  - `draw_rect()`: Draws filled rectangles
  - `draw_rect_outline()`: Draws rectangle borders
  - `set_projection()`: Sets the projection matrix

### Rendering Pipeline

1. Initialize renderer with shader and vertex data
2. Set projection matrix (typically orthographic for 2D UI)
3. For each frame:
   - Set position, size, and color uniforms
   - Draw quad using `gl::DrawArrays`

## Layer 3: UI Components (`src/ui/`)

This layer provides pre-built UI widgets ready to use.

### Common Interface: `Widget` Trait

All UI components implement the `Widget` trait:

```rust
pub trait Widget {
    fn draw(&self, renderer: &QuadRenderer);
    fn position(&self) -> Vec2;
    fn size(&self) -> Vec2;
    fn contains_point(&self, point: Vec2) -> bool;
}
```

### Components

#### `Label` (`label.rs`)
- Displays text (currently renders as a colored rectangle)
- Properties: position, size, text, color

#### `Button` (`button.rs`)
- Clickable button with state management
- States: normal, hover, pressed
- Properties: position, size, label, colors for each state

#### `Checkbox` (`checkbox.rs`)
- Toggle checkbox
- States: checked, unchecked
- Visual indicator when checked

#### `TextBox` (`textbox.rs`)
- Text input field
- States: focused, unfocused
- Basic text editing: insert char, backspace
- Cursor position tracking

#### `Panel` (`panel.rs`)
- Draggable container panel
- Features:
  - Title bar for dragging
  - Content area
  - Drag state management
- Methods for drag interaction: `start_drag()`, `update_drag()`, `stop_drag()`

### State Management

Each component manages its own state:
- Visual states (hover, focus, pressed)
- Content state (text, checked/unchecked)
- Interaction state (dragging, cursor position)

## Data Flow

### Rendering Flow
```
Application → Widget.draw() → QuadRenderer → Primitives → OpenGL
```

### Input Flow
```
Window Events → Application Logic → Widget State Updates → Re-render
```

## Coordinate System

- Origin (0, 0) at top-left corner
- X-axis increases to the right
- Y-axis increases downward
- Typically uses orthographic projection for 2D UI

## Extension Points

The library is designed to be easily extensible:

1. **New Primitives**: Add new OpenGL wrapper types in `src/primitives/`
2. **New Renderers**: Add specialized renderers in `src/renderer/`
3. **New Widgets**: Implement the `Widget` trait for new UI components
4. **Custom Shaders**: Create custom shaders for special rendering effects

## Future Enhancements

Potential areas for improvement:

1. **Text Rendering**: Integrate a text rendering library (e.g., rusttype, freetype)
2. **Texture Support**: Add texture rendering to widgets
3. **Layout System**: Add automatic layout management
4. **Event System**: Create a proper event handling system
5. **Theming**: Add a theme system for consistent styling
6. **Accessibility**: Add accessibility features
7. **Performance**: Batch rendering for multiple widgets

## Dependencies

- **gl**: OpenGL bindings
- **glam**: Mathematics library for vectors and matrices
- **glfw** (dev-dependency): Window management for examples

## Testing Strategy

The library uses integration tests to verify UI component behavior:
- State management tests
- Interaction tests (clicking, dragging, typing)
- Hit detection tests
- Widget creation and configuration tests

No OpenGL context is required for these tests as they only test the logical behavior of components.
