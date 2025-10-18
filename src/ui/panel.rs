use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, MouseButton, UiEvent, Widget, WidgetEvent};
use glam::{Vec2, Vec4};

/// A draggable panel that can contain other widgets
pub struct Panel {
    position: Vec2,
    size: Vec2,
    title: String,
    title_bar_height: f32,
    background_color: Vec4,
    title_bar_color: Vec4,
    border_color: Vec4,
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
            background_color: translucent(colors::SURFACE_DARK, 0.72),
            title_bar_color: translucent(colors::ACCENT, 0.9),
            border_color: colors::BORDER_SOFT,
            is_dragging: false,
            drag_offset: Vec2::ZERO,
        }
    }

    /// Sets the panel colors
    pub fn with_colors(mut self, background: Vec4, title_bar: Vec4) -> Self {
        self.background_color = translucent(background, 0.72);
        self.title_bar_color = translucent(title_bar, 0.9);
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
        let shadow_offset = Vec2::new(3.0, 4.0);
        renderer.draw_rect(self.position + shadow_offset, self.size, colors::SHADOW);

        // Draw title bar
        let title_bar_pos = self.position;
        let title_bar_size = Vec2::new(self.size.x, self.title_bar_height);
        renderer.draw_rect(title_bar_pos, title_bar_size, self.title_bar_color);
        renderer.draw_rect(
            title_bar_pos,
            Vec2::new(self.size.x, (self.title_bar_height * 0.6).max(1.0)),
            Vec4::new(1.0, 1.0, 1.0, 0.16),
        );
        // Title text centered in title bar
        let text_size = renderer.measure_text(&self.title);
        let text_pos = Vec2::new(
            self.position.x + (self.size.x - text_size.x) * 0.5,
            self.position.y + (self.title_bar_height - text_size.y) * 0.5,
        );
        renderer.draw_text(text_pos, colors::TEXT_PRIMARY, &self.title);

        // Draw panel background
        let panel_pos = Vec2::new(self.position.x, self.position.y + self.title_bar_height);
        let panel_size = Vec2::new(self.size.x, self.size.y - self.title_bar_height);
        renderer.draw_rect(panel_pos, panel_size, self.background_color);
        renderer.draw_rect(
            panel_pos,
            Vec2::new(self.size.x, (panel_size.y * 0.3).max(1.0)),
            Vec4::new(1.0, 1.0, 1.0, 0.06),
        );

        // Draw border around the entire panel
        renderer.draw_rect_outline(self.position, self.size, self.border_color, 2.0);
        if self.size.x > 6.0 && self.size.y > 6.0 {
            renderer.draw_rect_outline(
                self.position + Vec2::splat(2.0),
                self.size - Vec2::splat(4.0),
                colors::BORDER_SUBTLE,
                1.0,
            );
        }

        // Draw line separating title bar from content
        let separator_pos = Vec2::new(self.position.x, self.position.y + self.title_bar_height);
        let separator_size = Vec2::new(self.size.x, 2.0);
        renderer.draw_rect(separator_pos, separator_size, colors::BORDER_SOFT);
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        match event {
            UiEvent::CursorMoved { position } => {
                if self.is_dragging {
                    self.update_drag(*position);
                    Some(WidgetEvent::PanelDragged {
                        position: self.position,
                    })
                } else {
                    None
                }
            }
            UiEvent::MouseButton {
                button,
                state,
                position,
            } => {
                if *button != MouseButton::Left {
                    return None;
                }
                match state {
                    ButtonState::Pressed => {
                        if self.title_bar_contains_point(*position) {
                            self.start_drag(*position);
                            Some(WidgetEvent::PanelDragStarted)
                        } else {
                            None
                        }
                    }
                    ButtonState::Released => {
                        if self.is_dragging {
                            self.stop_drag();
                            Some(WidgetEvent::PanelDragEnded)
                        } else {
                            None
                        }
                    }
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

fn translucent(color: Vec4, fallback_alpha: f32) -> Vec4 {
    let alpha = if color.w <= 0.0 || color.w >= 0.99 {
        fallback_alpha
    } else {
        color.w
    };
    Vec4::new(color.x, color.y, color.z, alpha.clamp(0.45, 0.92))
}
