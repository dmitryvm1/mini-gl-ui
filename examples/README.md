# Examples

## Demo Example

The `demo.rs` example demonstrates all UI components in the mini-gl-ui library:
- Label
- Button (with hover and click states)
- Checkbox (toggle on/off)
- TextBox (with focus and text input)
- Draggable & collapsible Panel (+/- toggle)

### Running the Demo

**Note**: The demo requires X11 libraries on Linux systems.

#### Prerequisites

On Ubuntu/Debian:
```bash
sudo apt-get install libx11-dev libxrandr-dev libxi-dev
```

On Fedora:
```bash
sudo dnf install libX11-devel libXrandr-devel libXi-devel
```

On macOS:
```bash
# No additional dependencies needed
```

On Windows:
```bash
# No additional dependencies needed
```

#### Building and Running

```bash
cargo run --example demo
```

### Features Demonstrated

1. **Button Interaction**: Click the button to see console output
2. **Checkbox Toggle**: Click the checkbox to toggle its state
3. **TextBox Focus**: Click the textbox to focus it, then type keys (a-e, space, backspace)
4. **Panel Dragging & Collapse**: Click and drag the panel's title bar to move it or tap the +/- icon to collapse/expand its contents
5. **Visual Feedback**: Hover effects on buttons, focus states on textboxes

### Controls

- **Mouse**: Click and drag to interact with UI elements
- **Keyboard**: Type when textbox is focused (limited key support in demo)
- **ESC**: Close the window
