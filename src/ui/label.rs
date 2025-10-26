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

    /// Sets the label size
    pub fn set_size(&mut self, size: Vec2) {
        self.size = Vec2::new(size.x.max(0.0), size.y.max(0.0));
    }

    /// Sets the label position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }
}

impl Widget for Label {
    fn draw(&self, renderer: &QuadRenderer) {
        let fill = translucent(self.color, 0.68);
        let shadow_offset = Vec2::new(1.0, 2.0);
        renderer.draw_rect(self.position + shadow_offset, self.size, colors::SHADOW);
        renderer.draw_rect(self.position, self.size, fill);
        let highlight_height = (self.size.y * 0.35).max(1.0);
        renderer.draw_rect(
            self.position,
            Vec2::new(self.size.x, highlight_height),
            Vec4::new(1.0, 1.0, 1.0, 0.08),
        );
        renderer.draw_rect_outline(self.position, self.size, colors::BORDER_SOFT, 1.5);
        if self.size.x > 4.0 && self.size.y > 4.0 {
            renderer.draw_rect_outline(
                self.position + Vec2::splat(1.5),
                self.size - Vec2::splat(3.0),
                colors::BORDER_SUBTLE,
                1.0,
            );
        }
        // Text (top-left with small padding)
        let padding = 6.0;
        let text_pos = Vec2::new(self.position.x + padding, self.position.y);
        renderer.draw_text(text_pos, readable_text_color(fill), &self.text);
    }

    fn type_name(&self) -> &'static str {
        "Label"
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }
}

impl crate::ui::LayoutElement for Label {
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
    Vec4::new(color.x, color.y, color.z, alpha.clamp(0.45, 0.9))
}

fn readable_text_color(background: Vec4) -> Vec4 {
    let luminance = background.x * 0.299 + background.y * 0.587 + background.z * 0.114;
    if luminance > 0.55 {
        colors::BLACK
    } else {
        colors::TEXT_PRIMARY
    }
}
