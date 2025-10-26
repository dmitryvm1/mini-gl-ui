use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, MouseButton, UiEvent, Widget, WidgetEvent};
use glam::{Vec2, Vec4};

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

    /// Sets the checkbox position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Sets the checkbox size
    pub fn set_size(&mut self, size: Vec2) {
        self.size = Vec2::new(size.x.max(0.0), size.y.max(0.0));
    }

    /// Sets the checkbox label
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Toggles the checkbox
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

impl Widget for Checkbox {
    fn draw(&self, renderer: &QuadRenderer) {
        let box_side = self.size.x.min(self.size.y);
        let box_size = Vec2::splat(box_side);
        // Drop shadow softens the element over the scene
        let shadow_offset = Vec2::new(1.5, 2.5);
        renderer.draw_rect(self.position + shadow_offset, box_size, colors::SHADOW);
        renderer.draw_rect(self.position, box_size, colors::SURFACE);
        let highlight_height = (box_size.y * 0.4).max(1.0);
        renderer.draw_rect(
            self.position,
            Vec2::new(box_size.x, highlight_height),
            Vec4::new(1.0, 1.0, 1.0, 0.1),
        );
        renderer.draw_rect_outline(self.position, box_size, colors::BORDER_SOFT, 2.0);
        if box_size.x > 6.0 && box_size.y > 6.0 {
            renderer.draw_rect_outline(
                self.position + Vec2::splat(2.0),
                box_size - Vec2::splat(4.0),
                colors::BORDER_SUBTLE,
                1.0,
            );
        }

        // Draw check mark if checked
        if self.checked {
            let inset = (box_side * 0.24).clamp(3.0, 6.0);
            let check_pos = self.position + Vec2::splat(inset);
            let check_extent = (box_side - inset * 2.0).max(2.0);
            let check_size = Vec2::splat(check_extent);
            renderer.draw_rect(check_pos, check_size, colors::CHECKMARK);
            renderer.draw_rect_outline(check_pos, check_size, colors::BORDER_SOFT, 1.0);
        }

        // Draw label to the right of the box
        let spacing = 8.0;
        let text_pos = {
            let baseline_origin =
                Vec2::new(self.position.x + box_size.x + spacing, self.position.y);
            let measured = renderer.measure_text(&self.label);
            if measured == Vec2::ZERO {
                baseline_origin
            } else {
                Vec2::new(
                    baseline_origin.x,
                    baseline_origin.y + (self.size.y - measured.y) * 0.5,
                )
            }
        };
        renderer.draw_text(text_pos, colors::TEXT_PRIMARY, &self.label);
    }

    fn type_name(&self) -> &'static str {
        "Checkbox"
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        match event {
            UiEvent::MouseButton {
                button,
                state,
                position,
            } => {
                if *button == MouseButton::Left
                    && *state == ButtonState::Pressed
                    && self.contains_point(*position)
                {
                    self.toggle();
                    Some(WidgetEvent::CheckboxToggled {
                        label: self.label.clone(),
                        checked: self.checked,
                    })
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

impl crate::ui::LayoutElement for Checkbox {
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
