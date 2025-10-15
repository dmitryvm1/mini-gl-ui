use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::Widget;
use glam::{Vec2, Vec4};

/// A clickable button
pub struct Button {
    position: Vec2,
    size: Vec2,
    label: String,
    normal_color: Vec4,
    hover_color: Vec4,
    pressed_color: Vec4,
    is_hovered: bool,
    is_pressed: bool,
}

impl Button {
    /// Creates a new button
    pub fn new(position: Vec2, size: Vec2, label: String) -> Self {
        Button {
            position,
            size,
            label,
            normal_color: colors::LIGHT_GRAY,
            hover_color: colors::GRAY,
            pressed_color: colors::DARK_GRAY,
            is_hovered: false,
            is_pressed: false,
        }
    }
    
    /// Sets the button colors
    pub fn with_colors(mut self, normal: Vec4, hover: Vec4, pressed: Vec4) -> Self {
        self.normal_color = normal;
        self.hover_color = hover;
        self.pressed_color = pressed;
        self
    }
    
    /// Gets the button label
    pub fn label(&self) -> &str {
        &self.label
    }
    
    /// Sets hover state
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }
    
    /// Sets pressed state
    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }
    
    /// Checks if button is pressed
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
    
    /// Gets the current button color based on state
    fn current_color(&self) -> Vec4 {
        if self.is_pressed {
            self.pressed_color
        } else if self.is_hovered {
            self.hover_color
        } else {
            self.normal_color
        }
    }
}

impl Widget for Button {
    fn draw(&self, renderer: &QuadRenderer) {
        let color = self.current_color();
        renderer.draw_rect(self.position, self.size, color);
        renderer.draw_rect_outline(self.position, self.size, colors::BLACK, 2.0);
    }
    
    fn position(&self) -> Vec2 {
        self.position
    }
    
    fn size(&self) -> Vec2 {
        self.size
    }
}
