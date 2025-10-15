//! UI components built on top of the rendering layer

mod label;
mod button;
mod checkbox;
mod textbox;
mod panel;

pub use label::Label;
pub use button::Button;
pub use checkbox::Checkbox;
pub use textbox::TextBox;
pub use panel::Panel;

use glam::Vec2;

/// Common trait for UI components
pub trait Widget {
    /// Draws the widget
    fn draw(&self, renderer: &crate::renderer::QuadRenderer);
    
    /// Returns the position of the widget
    fn position(&self) -> Vec2;
    
    /// Returns the size of the widget
    fn size(&self) -> Vec2;
    
    /// Checks if a point is inside the widget bounds
    fn contains_point(&self, point: Vec2) -> bool {
        let pos = self.position();
        let size = self.size();
        point.x >= pos.x && point.x <= pos.x + size.x && 
        point.y >= pos.y && point.y <= pos.y + size.y
    }
}
