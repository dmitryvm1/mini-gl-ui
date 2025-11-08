use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, MouseButton, UiEvent, Widget, WidgetEvent};
use glam::{Vec2, Vec4};

/// A clickable button
pub struct Button {
    id: String,
    position: Vec2,
    size: Vec2,
    label: String,
    normal_color_override: Option<Vec4>,
    hover_color_override: Option<Vec4>,
    pressed_color_override: Option<Vec4>,
    text_color_override: Option<Vec4>,
    border_color_override: Option<Vec4>,
    is_hovered: bool,
    is_pressed: bool,
}

impl Button {
    /// Creates a new button
    pub fn new(
        id: impl Into<String>,
        position: Vec2,
        size: Vec2,
        label: impl Into<String>,
    ) -> Self {
        Button {
            id: id.into(),
            position,
            size,
            label: label.into(),
            normal_color_override: None,
            hover_color_override: None,
            pressed_color_override: None,
            text_color_override: None,
            border_color_override: None,
            is_hovered: false,
            is_pressed: false,
        }
    }

    /// Sets the button colors
    pub fn with_colors(mut self, normal: Vec4, hover: Vec4, pressed: Vec4) -> Self {
        self.normal_color_override = Some(translucent(normal, 0.8));
        self.hover_color_override = Some(translucent(hover, 0.82));
        self.pressed_color_override = Some(translucent(pressed, 0.88));
        self
    }

    /// Gets the button label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the button position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Sets the button size
    pub fn set_size(&mut self, size: Vec2) {
        self.size = Vec2::new(size.x.max(0.0), size.y.max(0.0));
    }

    /// Sets the button label
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Updates the button colors
    pub fn set_colors(&mut self, normal: Vec4, hover: Vec4, pressed: Vec4) {
        self.normal_color_override = Some(translucent(normal, 0.8));
        self.hover_color_override = Some(translucent(hover, 0.82));
        self.pressed_color_override = Some(translucent(pressed, 0.88));
    }

    /// Sets the text color
    pub fn set_text_color(&mut self, color: Vec4) {
        self.text_color_override = Some(color);
    }

    /// Sets the border color
    pub fn set_border_color(&mut self, color: Vec4) {
        self.border_color_override = Some(color);
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
            self.pressed_color_override
                .unwrap_or_else(default_pressed_color)
        } else if self.is_hovered {
            self.hover_color_override
                .unwrap_or_else(default_hover_color)
        } else {
            self.normal_color_override
                .unwrap_or_else(default_normal_color)
        }
    }

    fn text_color(&self) -> Vec4 {
        self.text_color_override
            .unwrap_or_else(colors::text_primary)
    }

    fn border_color(&self) -> Vec4 {
        self.border_color_override
            .unwrap_or_else(colors::border_soft)
    }
}

impl Widget for Button {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, renderer: &QuadRenderer) {
        let color = self.current_color();
        // Soft drop shadow keeps background subtly visible underneath
        let shadow_offset = Vec2::new(2.0, 3.0);
        renderer.draw_rect(self.position + shadow_offset, self.size, colors::shadow());

        renderer.draw_rect(self.position, self.size, color);
        let highlight_alpha = if self.is_pressed { 0.08 } else { 0.14 };
        let highlight_height = (self.size.y * 0.45).max(1.0);
        renderer.draw_rect(
            self.position,
            Vec2::new(self.size.x, highlight_height),
            Vec4::new(1.0, 1.0, 1.0, highlight_alpha),
        );

        renderer.draw_rect_outline(self.position, self.size, self.border_color(), 2.0);
        if self.size.x > 4.0 && self.size.y > 4.0 {
            renderer.draw_rect_outline(
                self.position + Vec2::splat(2.0),
                self.size - Vec2::splat(4.0),
                colors::border_subtle(),
                1.0,
            );
        }
        // Centered text
        let text_size = renderer.measure_text(&self.label);
        let text_pos = Vec2::new(
            self.position.x + (self.size.x - text_size.x) * 0.5,
            self.position.y + (self.size.y - text_size.y) * 0.5,
        );
        renderer.draw_text(text_pos, self.text_color(), &self.label);
    }

    fn type_name(&self) -> &'static str {
        "Button"
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        match event {
            UiEvent::CursorMoved { position } => {
                self.set_hovered(self.contains_point(*position));
                None
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
                        if self.contains_point(*position) {
                            self.set_pressed(true);
                        }
                        None
                    }
                    ButtonState::Released => {
                        let was_pressed = self.is_pressed;
                        self.set_pressed(false);
                        if was_pressed && self.contains_point(*position) {
                            Some(WidgetEvent::ButtonClicked {
                                id: self.id.clone(),
                                label: self.label.clone(),
                            })
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

impl crate::ui::LayoutElement for Button {
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
    Vec4::new(color.x, color.y, color.z, alpha.clamp(0.45, 0.95))
}

fn default_normal_color() -> Vec4 {
    translucent(colors::surface_light(), 0.8)
}

fn default_hover_color() -> Vec4 {
    translucent(colors::accent_soft(), 0.82)
}

fn default_pressed_color() -> Vec4 {
    translucent(colors::accent(), 0.88)
}
