use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, LayoutElement, MouseButton, UiEvent, Widget, WidgetEvent};
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
    content_padding: Vec2,
    children: Vec<PanelChild>,
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
            content_padding: Vec2::splat(12.0),
            children: Vec::new(),
        }
    }

    /// Sets the panel colors
    pub fn with_colors(mut self, background: Vec4, title_bar: Vec4) -> Self {
        self.background_color = translucent(background, 0.72);
        self.title_bar_color = translucent(title_bar, 0.9);
        self
    }

    /// Sets padding applied inside the panel content area.
    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.content_padding = Vec2::new(padding.x.max(0.0), padding.y.max(0.0));
        self.sync_children_positions();
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
            self.sync_children_positions();
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
        if self.position != position {
            self.position = position;
            self.sync_children_positions();
        }
    }

    /// Returns the top-left position of the panel content area.
    pub fn content_origin(&self) -> Vec2 {
        Vec2::new(
            self.position.x + self.content_padding.x,
            self.position.y + self.title_bar_height + self.content_padding.y,
        )
    }

    /// Sets the panel size
    pub fn set_size(&mut self, size: Vec2) {
        let min_height = self.title_bar_height + 4.0;
        self.size = Vec2::new(size.x.max(0.0), size.y.max(min_height));
        self.sync_children_positions();
    }

    /// Sets the panel title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Updates the panel colors
    pub fn set_colors(&mut self, background: Vec4, title_bar: Vec4) {
        self.background_color = translucent(background, 0.72);
        self.title_bar_color = translucent(title_bar, 0.9);
    }

    /// Sets the border color
    pub fn set_border_color(&mut self, color: Vec4) {
        self.border_color = color;
    }

    /// Updates padding applied inside the panel content area
    pub fn set_padding(&mut self, padding: Vec2) {
        let new_padding = Vec2::new(padding.x.max(0.0), padding.y.max(0.0));
        if self.content_padding != new_padding {
            self.content_padding = new_padding;
            self.sync_children_positions();
        }
    }

    /// Adds a child widget to the panel at the given offset from the content origin.
    pub fn add_child(&mut self, child: impl LayoutElement + 'static, offset: Vec2) {
        let mut child: Box<dyn LayoutElement> = Box::new(child);
        let offset = Vec2::new(offset.x, offset.y);
        child.set_position(self.content_origin() + offset);
        self.children.push(PanelChild {
            widget: child,
            offset,
        });
    }

    /// Returns the number of child widgets.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns true when the panel has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Removes the child at the provided index and returns it.
    pub fn remove_child(&mut self, index: usize) -> Option<Box<dyn LayoutElement>> {
        if index < self.children.len() {
            let removed = self.children.remove(index);
            self.sync_children_positions();
            Some(removed.widget)
        } else {
            None
        }
    }

    /// Adds a child widget to the panel using builder-style API.
    pub fn with_child(mut self, child: impl LayoutElement + 'static, offset: Vec2) -> Self {
        self.add_child(child, offset);
        self
    }

    /// Removes all child widgets from the panel.
    pub fn clear_children(&mut self) {
        if !self.children.is_empty() {
            self.children.clear();
        }
    }

    /// Returns an immutable reference to a child by index.
    pub fn child(&self, index: usize) -> Option<&dyn LayoutElement> {
        self.children.get(index).map(|child| child.widget.as_ref())
    }

    /// Returns a mutable reference to a child by index.
    pub fn child_mut(&mut self, index: usize) -> Option<&mut dyn LayoutElement> {
        self.children
            .get_mut(index)
            .map(|child| child.widget.as_mut())
    }

    fn sync_children_positions(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let origin = self.content_origin();
        for child in &mut self.children {
            child.widget.set_position(origin + child.offset);
        }
    }

    fn dispatch_event_to_children(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        for child in self.children.iter_mut().rev() {
            if let Some(widget_event) = child.widget.handle_event(event) {
                return Some(widget_event);
            }
        }
        None
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

        // Draw line separating title bar from content
        let separator_pos = Vec2::new(self.position.x, self.position.y + self.title_bar_height);
        let separator_size = Vec2::new(self.size.x, 2.0);
        renderer.draw_rect(separator_pos, separator_size, colors::BORDER_SOFT);

        // Draw child widgets
        for child in &self.children {
            child.widget.draw(renderer);
        }

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
    }

    fn type_name(&self) -> &'static str {
        "Panel"
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        match event {
            UiEvent::CursorMoved { position } => {
                if self.is_dragging {
                    self.update_drag(*position);
                    return Some(WidgetEvent::PanelDragged {
                        position: self.position,
                    });
                }
                self.dispatch_event_to_children(event)
            }
            UiEvent::MouseButton {
                button,
                state,
                position,
            } => {
                if *button != MouseButton::Left {
                    return self.dispatch_event_to_children(event);
                }
                match state {
                    ButtonState::Pressed => {
                        if self.title_bar_contains_point(*position) {
                            self.start_drag(*position);
                            Some(WidgetEvent::PanelDragStarted)
                        } else {
                            self.dispatch_event_to_children(event)
                        }
                    }
                    ButtonState::Released => {
                        let child_event = self.dispatch_event_to_children(event);
                        if self.is_dragging {
                            self.stop_drag();
                            child_event.or(Some(WidgetEvent::PanelDragEnded))
                        } else {
                            child_event
                        }
                    }
                }
            }
            _ => self.dispatch_event_to_children(event),
        }
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn contains_point(&self, point: Vec2) -> bool {
        let within_panel = point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y;
        let mut child_contains = false;
        for c in &self.children {
            if c.widget.contains_point(point) {
                child_contains = true;
                println!("Child hit: {}", c.widget.type_name());
                break;
            }
        }
        within_panel
            || child_contains
    }
}

impl crate::ui::LayoutElement for Panel {
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
    Vec4::new(color.x, color.y, color.z, alpha.clamp(0.45, 0.92))
}

struct PanelChild {
    widget: Box<dyn LayoutElement>,
    offset: Vec2,
}
