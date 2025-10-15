use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::Widget;
use glam::{Vec2, Vec4};

/// A draggable panel that can contain other widgets
pub struct Panel {
    position: Vec2,
    size: Vec2,
    title: String,
    title_bar_height: f32,
    background_color: Vec4,
    title_bar_color: Vec4,
    is_dragging: bool,
    drag_offset: Vec2,
}

impl Panel {
    /// Creates a new panel
    pub fn new(position: Vec2, size: Vec2, title: String) -> Self {
        Panel {
            position,
            size,
            title,
            title_bar_height: 30.0,
            background_color: colors::LIGHT_GRAY,
            title_bar_color: colors::DARK_GRAY,
            is_dragging: false,
            drag_offset: Vec2::ZERO,
        }
    }
    
    /// Sets the panel colors
    pub fn with_colors(mut self, background: Vec4, title_bar: Vec4) -> Self {
        self.background_color = background;
        self.title_bar_color = title_bar;
        self
    }
    
    /// Gets the panel title
    pub fn title(&self) -> &str {
        &self.title
    }
    
    /// Checks if a point is in the title bar (for dragging)
    pub fn title_bar_contains_point(&self, point: Vec2) -> bool {
        point.x >= self.position.x 
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.title_bar_height
    }
    
    /// Starts dragging the panel
    pub fn start_drag(&mut self, mouse_pos: Vec2) {
        self.is_dragging = true;
        self.drag_offset = mouse_pos - self.position;
    }
    
    /// Updates the panel position while dragging
    pub fn update_drag(&mut self, mouse_pos: Vec2) {
        if self.is_dragging {
            self.position = mouse_pos - self.drag_offset;
        }
    }
    
    /// Stops dragging the panel
    pub fn stop_drag(&mut self) {
        self.is_dragging = false;
    }
    
    /// Checks if the panel is being dragged
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
    
    /// Sets the panel position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }
}

impl Widget for Panel {
    fn draw(&self, renderer: &QuadRenderer) {
        // Draw title bar
        let title_bar_pos = self.position;
        let title_bar_size = Vec2::new(self.size.x, self.title_bar_height);
        renderer.draw_rect(title_bar_pos, title_bar_size, self.title_bar_color);
        
        // Draw panel background
        let panel_pos = Vec2::new(self.position.x, self.position.y + self.title_bar_height);
        let panel_size = Vec2::new(self.size.x, self.size.y - self.title_bar_height);
        renderer.draw_rect(panel_pos, panel_size, self.background_color);
        
        // Draw border around the entire panel
        renderer.draw_rect_outline(self.position, self.size, colors::BLACK, 2.0);
        
        // Draw line separating title bar from content
        let separator_pos = Vec2::new(self.position.x, self.position.y + self.title_bar_height);
        let separator_size = Vec2::new(self.size.x, 2.0);
        renderer.draw_rect(separator_pos, separator_size, colors::BLACK);
    }
    
    fn position(&self) -> Vec2 {
        self.position
    }
    
    fn size(&self) -> Vec2 {
        self.size
    }
}
