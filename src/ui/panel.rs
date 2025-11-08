use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, Dropdown, LayoutElement, MouseButton, UiEvent, Widget, WidgetEvent};
use glam::{Vec2, Vec4};

/// A draggable panel that can contain other widgets and be collapsed or expanded.
pub struct Panel {
    id: String,
    position: Vec2,
    size: Vec2,
    expanded_size: Vec2,
    title: String,
    title_bar_height: f32,
    background_color_override: Option<Vec4>,
    title_bar_color_override: Option<Vec4>,
    border_color_override: Option<Vec4>,
    is_dragging: bool,
    is_collapsed: bool,
    drag_offset: Vec2,
    content_padding: Vec2,
    children: Vec<PanelChild>,
}

impl Panel {
    /// Creates a new panel
    pub fn new(
        id: impl Into<String>,
        position: Vec2,
        size: Vec2,
        title: impl Into<String>,
    ) -> Self {
        let title_bar_height = 30.0;
        let min_height = title_bar_height + 4.0;
        let expanded_size = Vec2::new(size.x.max(0.0), size.y.max(min_height));
        Panel {
            id: id.into(),
            position,
            size: expanded_size,
            expanded_size,
            title: title.into(),
            title_bar_height,
            background_color_override: None,
            title_bar_color_override: None,
            border_color_override: None,
            is_dragging: false,
            is_collapsed: false,
            drag_offset: Vec2::ZERO,
            content_padding: Vec2::splat(12.0),
            children: Vec::new(),
        }
    }

    /// Sets the panel colors
    pub fn with_colors(mut self, background: Vec4, title_bar: Vec4) -> Self {
        self.background_color_override = Some(translucent(background, 0.72));
        self.title_bar_color_override = Some(translucent(title_bar, 0.9));
        self
    }

    /// Sets padding applied inside the panel content area.
    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.content_padding = Vec2::new(padding.x.max(0.0), padding.y.max(0.0));
        self.sync_children_positions();
        self
    }

    /// Returns true when the panel is collapsed.
    pub fn is_collapsed(&self) -> bool {
        self.is_collapsed
    }

    /// Collapses or expands the panel content area.
    pub fn set_collapsed(&mut self, collapsed: bool) {
        if self.is_collapsed == collapsed {
            return;
        }
        self.is_collapsed = collapsed;
        self.apply_stateful_size();
        self.sync_children_positions();
    }

    /// Toggles the collapsed state of the panel.
    pub fn toggle_collapsed(&mut self) {
        let next_state = !self.is_collapsed;
        self.set_collapsed(next_state);
    }

    /// Height of the title bar region.
    pub fn title_bar_height(&self) -> f32 {
        self.title_bar_height
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
        self.expanded_size = Vec2::new(size.x.max(0.0), size.y.max(min_height));
        self.apply_stateful_size();
        self.sync_children_positions();
    }

    /// Sets the panel title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Updates the panel colors
    pub fn set_colors(&mut self, background: Vec4, title_bar: Vec4) {
        self.background_color_override = Some(translucent(background, 0.72));
        self.title_bar_color_override = Some(translucent(title_bar, 0.9));
    }

    /// Sets the border color
    pub fn set_border_color(&mut self, color: Vec4) {
        self.border_color_override = Some(color);
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

    fn apply_stateful_size(&mut self) {
        if self.is_collapsed {
            self.size = Vec2::new(self.expanded_size.x, self.title_bar_height);
        } else {
            self.size = self.expanded_size;
        }
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
        if self.is_collapsed {
            return None;
        }

        let capture_pointer = matches!(
            event,
            UiEvent::MouseButton {
                state: ButtonState::Pressed,
                ..
            } | UiEvent::Scroll { .. }
        );
        let pointer_position = match event {
            UiEvent::MouseButton { position, .. } => Some(*position),
            UiEvent::Scroll { position, .. } => Some(*position),
            _ => None,
        };

        let overlay_index = if capture_pointer {
            pointer_position.and_then(|position| {
                self.children
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, child)| {
                        child
                            .widget
                            .as_any()
                            .downcast_ref::<Dropdown>()
                            .filter(|dropdown| dropdown.overlay_contains_point(position))
                            .map(|_| index)
                    })
            })
        } else {
            None
        };

        if let Some(index) = overlay_index {
            if let Some(child) = self.children.get_mut(index) {
                if let Some(widget_event) = child.widget.handle_event(event) {
                    return Some(widget_event);
                }
            }
        }

        let mut pointer_claimed = capture_pointer && overlay_index.is_some();

        for index in (0..self.children.len()).rev() {
            if Some(index) == overlay_index {
                continue;
            }
            let child = &mut self.children[index];
            let hit = pointer_position
                .map_or(false, |position| child.widget.contains_point(position));

            if capture_pointer && pointer_claimed && hit {
                continue;
            }

            if let Some(widget_event) = child.widget.handle_event(event) {
                return Some(widget_event);
            }

            if capture_pointer && hit {
                pointer_claimed = true;
            }
        }
        None
    }

    fn toggle_button_bounds(&self) -> (Vec2, Vec2) {
        let margin_x = 10.0;
        let button_size = (self.title_bar_height * 0.6).clamp(14.0, self.title_bar_height - 6.0);
        let button_pos = Vec2::new(
            self.position.x + margin_x,
            self.position.y + (self.title_bar_height - button_size) * 0.5,
        );
        (button_pos, Vec2::splat(button_size))
    }

    fn toggle_button_contains_point(&self, point: Vec2) -> bool {
        let (pos, size) = self.toggle_button_bounds();
        point.x >= pos.x
            && point.x <= pos.x + size.x
            && point.y >= pos.y
            && point.y <= pos.y + size.y
    }

    fn background_color(&self) -> Vec4 {
        self.background_color_override
            .unwrap_or_else(default_panel_background_color)
    }

    fn title_bar_color(&self) -> Vec4 {
        self.title_bar_color_override
            .unwrap_or_else(default_panel_title_bar_color)
    }

    fn border_color(&self) -> Vec4 {
        self.border_color_override
            .unwrap_or_else(colors::border_soft)
    }

    fn draw_toggle_button(&self, renderer: &QuadRenderer, position: Vec2, size: Vec2) {
        let fill_color = if self.is_collapsed {
            translucent(colors::surface_light(), 0.75)
        } else {
            translucent(colors::surface(), 0.8)
        };
        renderer.draw_rect(position, size, fill_color);
        renderer.draw_rect_outline(position, size, colors::border_soft(), 1.0);

        let symbol = if self.is_collapsed { "+" } else { "-" };
        let symbol_size = renderer.measure_text(symbol);
        let symbol_pos = Vec2::new(
            position.x + (size.x - symbol_size.x) * 0.5,
            position.y + (size.y - symbol_size.y) * 0.5,
        );
        renderer.draw_text(symbol_pos, colors::text_primary(), symbol);
    }
}

impl Widget for Panel {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, renderer: &QuadRenderer) {
        let shadow_offset = Vec2::new(3.0, 4.0);
        renderer.draw_rect(self.position + shadow_offset, self.size, colors::shadow());

        // Draw title bar
        let title_bar_pos = self.position;
        let title_bar_size = Vec2::new(self.size.x, self.title_bar_height);
        renderer.draw_rect(title_bar_pos, title_bar_size, self.title_bar_color());
        renderer.draw_rect(
            title_bar_pos,
            Vec2::new(self.size.x, (self.title_bar_height * 0.6).max(1.0)),
            Vec4::new(1.0, 1.0, 1.0, 0.16),
        );
        let (toggle_pos, toggle_size) = self.toggle_button_bounds();
        self.draw_toggle_button(renderer, toggle_pos, toggle_size);
        // Title text centered in title bar
        let text_size = renderer.measure_text(&self.title);
        let desired_x = self.position.x + (self.size.x - text_size.x) * 0.5;
        let min_text_x = toggle_pos.x + toggle_size.x + 8.0;
        let text_pos = Vec2::new(
            desired_x.max(min_text_x),
            self.position.y + (self.title_bar_height - text_size.y) * 0.5,
        );
        renderer.draw_text(text_pos, colors::text_primary(), &self.title);

        let content_height = (self.size.y - self.title_bar_height).max(0.0);
        if content_height > 0.5 {
            // Draw panel background
            let panel_pos = Vec2::new(self.position.x, self.position.y + self.title_bar_height);
            let panel_size = Vec2::new(self.size.x, content_height);
            renderer.draw_rect(panel_pos, panel_size, self.background_color());
            renderer.draw_rect(
                panel_pos,
                Vec2::new(self.size.x, (panel_size.y * 0.3).max(1.0)),
                Vec4::new(1.0, 1.0, 1.0, 0.06),
            );

            // Draw line separating title bar from content
            let separator_pos = Vec2::new(self.position.x, self.position.y + self.title_bar_height);
            let separator_size = Vec2::new(self.size.x, 2.0);
            renderer.draw_rect(separator_pos, separator_size, colors::border_soft());

            // Draw child widgets
            for child in &self.children {
                child.widget.draw(renderer);
            }
        }

        // Draw border around the entire panel
        renderer.draw_rect_outline(self.position, self.size, self.border_color(), 2.0);
        if self.size.x > 6.0 && self.size.y > 6.0 {
            renderer.draw_rect_outline(
                self.position + Vec2::splat(2.0),
                self.size - Vec2::splat(4.0),
                colors::border_subtle(),
                1.0,
            );
        }
    }

    fn draw_overlay(&self, renderer: &QuadRenderer) {
        if self.is_collapsed {
            return;
        }
        let content_height = (self.size.y - self.title_bar_height).max(0.0);
        if content_height <= 0.5 {
            return;
        }
        for child in &self.children {
            child.widget.draw_overlay(renderer);
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
                        id: self.id.clone(),
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
                        if self.toggle_button_contains_point(*position) {
                            self.toggle_collapsed();
                            return Some(WidgetEvent::PanelToggleChanged {
                                id: self.id.clone(),
                                collapsed: self.is_collapsed,
                            });
                        }
                        if self.title_bar_contains_point(*position) {
                            self.start_drag(*position);
                            Some(WidgetEvent::PanelDragStarted {
                                id: self.id.clone(),
                            })
                        } else {
                            self.dispatch_event_to_children(event)
                        }
                    }
                    ButtonState::Released => {
                        let child_event = self.dispatch_event_to_children(event);
                        if self.is_dragging {
                            self.stop_drag();
                            child_event.or(Some(WidgetEvent::PanelDragEnded {
                                id: self.id.clone(),
                            }))
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
        let child_contains = if self.is_collapsed {
            false
        } else {
            self.children
                .iter()
                .any(|child| child.widget.contains_point(point))
        };
        within_panel || child_contains
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

fn default_panel_background_color() -> Vec4 {
    translucent(colors::surface_dark(), 0.72)
}

fn default_panel_title_bar_color() -> Vec4 {
    translucent(colors::accent(), 0.9)
}

struct PanelChild {
    widget: Box<dyn LayoutElement>,
    offset: Vec2,
}
