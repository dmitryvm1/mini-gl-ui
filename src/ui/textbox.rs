use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::Widget;
use glam::Vec2;

/// A text input box
pub struct TextBox {
    position: Vec2,
    size: Vec2,
    text: String,
    _placeholder: String,
    is_focused: bool,
    cursor_position: usize,
}

impl TextBox {
    /// Creates a new text box
    pub fn new(position: Vec2, size: Vec2, placeholder: String) -> Self {
        TextBox {
            position,
            size,
            text: String::new(),
            _placeholder: placeholder,
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
    
    /// Checks if the textbox is focused
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
}

impl Widget for TextBox {
    fn draw(&self, renderer: &QuadRenderer) {
        // Draw background
        let bg_color = if self.is_focused {
            colors::WHITE
        } else {
            colors::LIGHT_GRAY
        };
        renderer.draw_rect(self.position, self.size, bg_color);
        
        // Draw border
        let border_color = if self.is_focused {
            colors::BLUE
        } else {
            colors::BLACK
        };
        renderer.draw_rect_outline(self.position, self.size, border_color, 2.0);
        
        // Draw text content
        let padding = 6.0;
        let text_pos = Vec2::new(self.position.x + padding, self.position.y + padding);
        let text_color = colors::BLACK;
        if !self.text.is_empty() {
            renderer.draw_text(text_pos, text_color, &self.text);
        }
    }
    
    fn position(&self) -> Vec2 {
        self.position
    }
    
    fn size(&self) -> Vec2 {
        self.size
    }
}
