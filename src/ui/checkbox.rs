use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::Widget;
use glam::Vec2;

/// A checkbox that can be checked or unchecked
pub struct Checkbox {
    position: Vec2,
    size: Vec2,
    checked: bool,
    label: String,
}

impl Checkbox {
    /// Creates a new checkbox
    pub fn new(position: Vec2, size: Vec2, label: String) -> Self {
        Checkbox {
            position,
            size,
            checked: false,
            label,
        }
    }
    
    /// Gets the checkbox label
    pub fn label(&self) -> &str {
        &self.label
    }
    
    /// Checks if the checkbox is checked
    pub fn is_checked(&self) -> bool {
        self.checked
    }
    
    /// Sets the checked state
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }
    
    /// Toggles the checkbox
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

impl Widget for Checkbox {
    fn draw(&self, renderer: &QuadRenderer) {
        // Draw the checkbox box
        renderer.draw_rect(self.position, self.size, colors::WHITE);
        renderer.draw_rect_outline(self.position, self.size, colors::BLACK, 2.0);
        
        // Draw check mark if checked
        if self.checked {
            let padding = 4.0;
            let check_pos = Vec2::new(
                self.position.x + padding,
                self.position.y + padding,
            );
            let check_size = Vec2::new(
                self.size.x - padding * 2.0,
                self.size.y - padding * 2.0,
            );
            renderer.draw_rect(check_pos, check_size, colors::GREEN);
        }
    }
    
    fn position(&self) -> Vec2 {
        self.position
    }
    
    fn size(&self) -> Vec2 {
        self.size
    }
}
