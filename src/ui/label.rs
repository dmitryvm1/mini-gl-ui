use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::Widget;
use glam::{Vec2, Vec4};

/// A simple text label (renders as a colored box for now)
pub struct Label {
    position: Vec2,
    size: Vec2,
    color: Vec4,
    text: String,
}

impl Label {
    /// Creates a new label
    pub fn new(position: Vec2, size: Vec2, text: String, color: Vec4) -> Self {
        Label {
            position,
            size,
            color,
            text,
        }
    }
    
    /// Gets the label text
    pub fn text(&self) -> &str {
        &self.text
    }
    
    /// Sets the label text
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
    
    /// Sets the label color
    pub fn set_color(&mut self, color: Vec4) {
        self.color = color;
    }
}

impl Widget for Label {
    fn draw(&self, renderer: &QuadRenderer) {
        // Background
        renderer.draw_rect(self.position, self.size, self.color);
        // Text (top-left with small padding)
        let padding = 6.0;
        let text_pos = Vec2::new(self.position.x + padding, self.position.y + padding);
        renderer.draw_text(text_pos, colors::BLACK, &self.text);
    }
    
    fn position(&self) -> Vec2 {
        self.position
    }
    
    fn size(&self) -> Vec2 {
        self.size
    }
}
