use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, KeyCode, MouseButton, UiEvent, Widget, WidgetEvent};
use glam::{Vec2, Vec4};

/// A text input box
pub struct TextBox {
    id: String,
    position: Vec2,
    size: Vec2,
    text: String,
    _placeholder: String,
    is_focused: bool,
    cursor_position: usize,
}

impl TextBox {
    /// Creates a new text box
    pub fn new(
        id: impl Into<String>,
        position: Vec2,
        size: Vec2,
        placeholder: impl Into<String>,
    ) -> Self {
        TextBox {
            id: id.into(),
            position,
            size,
            text: String::new(),
            _placeholder: placeholder.into(),
            is_focused: false,
            cursor_position: 0,
        }
    }

    /// Gets the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Sets the text content
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_position = self.text.len();
    }

    /// Appends a character to the text
    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Removes the character before the cursor
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
        }
    }

    /// Sets the focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Sets the textbox position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Sets the textbox size
    pub fn set_size(&mut self, size: Vec2) {
        self.size = Vec2::new(size.x.max(0.0), size.y.max(0.0));
    }

    /// Sets the placeholder text
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self._placeholder = placeholder.into();
    }

    /// Checks if the textbox is focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
}

impl Widget for TextBox {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, renderer: &QuadRenderer) {
        let base_bg = if self.is_focused {
            colors::SURFACE_LIGHT
        } else {
            colors::SURFACE
        };
        let bg_color = translucent(base_bg, if self.is_focused { 0.82 } else { 0.76 });

        renderer.draw_rect(
            self.position + Vec2::new(1.5, 3.0),
            self.size,
            colors::SHADOW,
        );
        renderer.draw_rect(self.position, self.size, bg_color);
        let highlight_height = (self.size.y * 0.35).max(1.0);
        let highlight_alpha = if self.is_focused { 0.12 } else { 0.08 };
        renderer.draw_rect(
            self.position,
            Vec2::new(self.size.x, highlight_height),
            Vec4::new(1.0, 1.0, 1.0, highlight_alpha),
        );

        // Draw border
        let border_color = if self.is_focused {
            colors::ACCENT
        } else {
            colors::BORDER_SOFT
        };
        renderer.draw_rect_outline(self.position, self.size, border_color, 2.0);
        if self.size.x > 4.0 && self.size.y > 4.0 {
            renderer.draw_rect_outline(
                self.position + Vec2::splat(2.0),
                self.size - Vec2::splat(4.0),
                colors::BORDER_SUBTLE,
                1.0,
            );
        }

        // Draw text content (vertically centered)
        let padding = 6.0;
        let text_color = colors::TEXT_PRIMARY;
        if !self.text.is_empty() {
            let x = self.position.x + padding;
            // Align by baseline using font line metrics to avoid vertical jumps
            if let Some((ascent, descent)) = renderer.line_metrics() {
                let center = self.position.y + self.size.y * 0.5;
                let baseline = center + (ascent - descent) * 0.5;
                let top = baseline - ascent;
                renderer.draw_text(Vec2::new(x, top), text_color, &self.text);
            } else {
                // Fallback to simple vertical centering when no font configured
                let text_size = renderer.measure_text(&self.text);
                let y = self.position.y + (self.size.y - text_size.y) * 0.5;
                renderer.draw_text(Vec2::new(x, y), text_color, &self.text);
            }
        }
    }

    fn type_name(&self) -> &'static str {
        "TextBox"
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        match event {
            UiEvent::MouseButton {
                button,
                state,
                position,
            } => {
                if *button == MouseButton::Left && *state == ButtonState::Pressed {
                    let new_focus = self.contains_point(*position);
                    let focus_changed = new_focus != self.is_focused;
                    self.set_focused(new_focus);
                    if focus_changed {
                        return Some(WidgetEvent::TextBoxFocusChanged {
                            id: self.id.clone(),
                            focused: new_focus,
                        });
                    }
                }
                None
            }
            UiEvent::CharacterInput(ch) => {
                if self.is_focused {
                    self.insert_char(*ch);
                    Some(WidgetEvent::TextChanged {
                        id: self.id.clone(),
                        text: self.text.clone(),
                    })
                } else {
                    None
                }
            }
            UiEvent::KeyInput { key } => {
                if self.is_focused {
                    match key {
                        KeyCode::Backspace => {
                            self.backspace();
                            Some(WidgetEvent::TextChanged {
                                id: self.id.clone(),
                                text: self.text.clone(),
                            })
                        }
                        KeyCode::Other => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }
}

impl crate::ui::LayoutElement for TextBox {
    fn set_position(&mut self, position: Vec2) {
        self.set_position(position);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn translucent(color: Vec4, fallback_alpha: f32) -> Vec4 {
    let alpha = if color.w <= 0.0 || color.w >= 0.99 {
        fallback_alpha
    } else {
        color.w
    };
    Vec4::new(color.x, color.y, color.z, alpha.clamp(0.45, 0.92))
}
