use crate::colors;
use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, MouseButton, UiEvent, Widget, WidgetEvent};
use glam::{Vec2, Vec4};

/// A dropdown widget that allows selecting from a scrollable list of options
pub struct Dropdown {
    position: Vec2,
    size: Vec2,
    id: String,
    options: Vec<String>,
    placeholder: Option<String>,
    selected_index: Option<usize>,
    is_open: bool,
    hovered_index: Option<usize>,
    scroll_offset: usize,
    max_visible_items: usize,
    option_height: f32,
    main_hovered: bool,
}

impl Dropdown {
    /// Creates a new dropdown with an identifier and initial options
    pub fn new(position: Vec2, size: Vec2, id: String, options: Vec<String>) -> Self {
        let selected_index = if options.is_empty() { None } else { Some(0) };
        Dropdown {
            position,
            size,
            id,
            options,
            placeholder: None,
            selected_index,
            is_open: false,
            hovered_index: None,
            scroll_offset: 0,
            max_visible_items: 5,
            option_height: 28.0,
            main_hovered: false,
        }
    }

    /// Sets placeholder text displayed when no option is selected
    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = Some(placeholder);
        self.selected_index = None;
        self.scroll_offset = 0;
        self
    }

    /// Sets the maximum number of visible items before scrolling is required
    pub fn with_max_visible_items(mut self, count: usize) -> Self {
        self.max_visible_items = count.max(1);
        self.clamp_scroll();
        self
    }

    /// Returns the currently selected option text
    pub fn selected(&self) -> Option<&str> {
        self.selected_index
            .and_then(|index| self.options.get(index).map(|s| s.as_str()))
    }

    /// Sets the selected option by index. Ignores out-of-range indices.
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = Some(index);
            self.ensure_selected_visible();
        }
    }

    /// Replaces the list of options and keeps the closest valid selection
    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
        if self.options.is_empty() {
            self.selected_index = None;
        } else {
            if let Some(selected) = self.selected_index {
                self.selected_index = Some(selected.min(self.options.len() - 1));
            }
        }
        self.scroll_offset = 0;
        self.ensure_selected_visible();
    }

    /// Returns true if the dropdown panel is currently open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Sets the dropdown position
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    fn visible_range(&self) -> (usize, usize) {
        if self.options.is_empty() {
            return (0, 0);
        }
        let visible = self.max_visible_items.min(self.options.len());
        let max_offset = self.max_scroll_offset();
        let start = self.scroll_offset.min(max_offset);
        let end = (start + visible).min(self.options.len());
        (start, end)
    }

    fn max_scroll_offset(&self) -> usize {
        let visible = self.max_visible_items.min(self.options.len());
        self.options.len().saturating_sub(visible.max(1))
    }

    fn clamp_scroll(&mut self) {
        let max_offset = self.max_scroll_offset();
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    fn ensure_selected_visible(&mut self) {
        self.clamp_scroll();
        if let Some(sel) = self.selected_index {
            let visible = self.max_visible_items.min(self.options.len());
            if visible == 0 {
                self.scroll_offset = 0;
                return;
            }
            if sel < self.scroll_offset {
                self.scroll_offset = sel;
            } else if sel >= self.scroll_offset + visible {
                self.scroll_offset = sel + 1 - visible;
            }
            self.clamp_scroll();
        }
    }

    fn main_bounds(&self) -> (Vec2, Vec2) {
        (self.position, self.size)
    }

    fn list_bounds(&self) -> Option<(Vec2, Vec2)> {
        if !self.is_open {
            return None;
        }
        let (start, end) = self.visible_range();
        if end <= start {
            return None;
        }
        let height = (end - start) as f32 * self.option_height;
        if height <= 0.0 {
            return None;
        }
        Some((
            Vec2::new(self.position.x, self.position.y + self.size.y),
            Vec2::new(self.size.x, height),
        ))
    }

    fn main_area_contains(&self, point: Vec2) -> bool {
        let (pos, size) = self.main_bounds();
        point.x >= pos.x
            && point.x <= pos.x + size.x
            && point.y >= pos.y
            && point.y <= pos.y + size.y
    }

    fn list_area_contains(&self, point: Vec2) -> bool {
        if let Some((pos, size)) = self.list_bounds() {
            return point.x >= pos.x
                && point.x <= pos.x + size.x
                && point.y >= pos.y
                && point.y <= pos.y + size.y;
        }
        false
    }

    fn update_hover_from_point(&mut self, point: Vec2) {
        if !self.is_open {
            self.hovered_index = None;
            return;
        }
        if let Some(index) = self.index_at_point(point) {
            self.hovered_index = Some(index);
        } else {
            self.hovered_index = None;
        }
    }

    fn index_at_point(&self, point: Vec2) -> Option<usize> {
        let (list_pos, _) = self.list_bounds()?;
        if !self.list_area_contains(point) {
            return None;
        }
        let relative_y = point.y - list_pos.y;
        if relative_y < 0.0 {
            return None;
        }
        let item = (relative_y / self.option_height).floor() as usize;
        let (start, end) = self.visible_range();
        let index = start + item;
        if index < end {
            Some(index)
        } else {
            None
        }
    }

    fn open(&mut self) {
        if self.options.is_empty() {
            return;
        }
        self.is_open = true;
        self.ensure_selected_visible();
    }

    fn close(&mut self) {
        self.is_open = false;
        self.hovered_index = None;
    }

    fn apply_selection(&mut self, index: usize) -> Option<WidgetEvent> {
        if index >= self.options.len() {
            self.close();
            return None;
        }
        let changed = Some(index) != self.selected_index;
        self.selected_index = Some(index);
        self.ensure_selected_visible();
        self.close();
        if changed {
            if let Some(selected) = self.selected() {
                return Some(WidgetEvent::DropdownSelectionChanged {
                    id: self.id.clone(),
                    selected: selected.to_string(),
                });
            }
        }
        None
    }
}

impl Widget for Dropdown {
    fn draw(&self, renderer: &QuadRenderer) {
        let (main_pos, main_size) = self.main_bounds();
        let shadow_offset = Vec2::new(2.0, 3.0);
        renderer.draw_rect(main_pos + shadow_offset, main_size, colors::SHADOW);

        let base_bg = translucent(colors::SURFACE, 0.78);
        let hover_bg = translucent(colors::SURFACE_LIGHT, 0.82);
        let open_bg = translucent(colors::ACCENT_SOFT, 0.82);
        let bg_color = if self.is_open {
            open_bg
        } else if self.main_hovered {
            hover_bg
        } else {
            base_bg
        };

        renderer.draw_rect(main_pos, main_size, bg_color);
        let highlight_height = (main_size.y * 0.35).max(1.0);
        renderer.draw_rect(
            main_pos,
            Vec2::new(main_size.x, highlight_height),
            Vec4::new(1.0, 1.0, 1.0, if self.is_open { 0.16 } else { 0.12 }),
        );

        renderer.draw_rect_outline(main_pos, main_size, colors::BORDER_SOFT, 2.0);
        if main_size.x > 4.0 && main_size.y > 4.0 {
            renderer.draw_rect_outline(
                main_pos + Vec2::splat(2.0),
                main_size - Vec2::splat(4.0),
                colors::BORDER_SUBTLE,
                1.0,
            );
        }

        let display_text = self
            .selected()
            .map(|s| s.to_string())
            .or_else(|| self.placeholder.clone())
            .unwrap_or_else(|| "Select...".to_string());
        let text_color = if self.selected_index.is_some() {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_SECONDARY
        };
        let padding = 10.0;
        let text_size = renderer.measure_text(&display_text);
        let text_pos = Vec2::new(
            main_pos.x + padding,
            main_pos.y + (main_size.y - text_size.y) * 0.5,
        );
        renderer.draw_text(text_pos, text_color, &display_text);

        let arrow_text = if self.is_open { "^" } else { "v" };
        let arrow_size = renderer.measure_text(arrow_text);
        let arrow_pos = Vec2::new(
            main_pos.x + main_size.x - arrow_size.x - padding,
            main_pos.y + (main_size.y - arrow_size.y) * 0.5,
        );
        renderer.draw_text(arrow_pos, colors::TEXT_SECONDARY, arrow_text);

        if let Some((list_pos, list_size)) = self.list_bounds() {
            renderer.draw_rect(list_pos + shadow_offset, list_size, colors::SHADOW);

            let panel_bg = translucent(colors::SURFACE_DARK, 0.78);
            renderer.draw_rect(list_pos, list_size, panel_bg);
            renderer.draw_rect_outline(list_pos, list_size, colors::BORDER_SOFT, 2.0);
            if list_size.x > 4.0 && list_size.y > 4.0 {
                renderer.draw_rect_outline(
                    list_pos + Vec2::splat(2.0),
                    list_size - Vec2::splat(4.0),
                    colors::BORDER_SUBTLE,
                    1.0,
                );
            }

            let (start, end) = self.visible_range();
            for (i, option_index) in (start..end).enumerate() {
                let item_pos = Vec2::new(list_pos.x, list_pos.y + i as f32 * self.option_height);
                let item_size = Vec2::new(self.size.x, self.option_height);
                let is_selected = Some(option_index) == self.selected_index;
                let is_hovered = Some(option_index) == self.hovered_index;
                let item_color = if is_selected {
                    translucent(colors::ACCENT, 0.82)
                } else if is_hovered {
                    translucent(colors::SURFACE_LIGHT, 0.86)
                } else {
                    translucent(colors::SURFACE, 0.82)
                };

                renderer.draw_rect(item_pos, item_size, item_color);
                if is_selected {
                    renderer.draw_rect(
                        item_pos,
                        Vec2::new(item_size.x, (item_size.y * 0.3).max(1.0)),
                        Vec4::new(1.0, 1.0, 1.0, 0.12),
                    );
                }

                if let Some(option) = self.options.get(option_index) {
                    let option_text = option.as_str();
                    let option_size = renderer.measure_text(option_text);
                    let option_pos = Vec2::new(
                        item_pos.x + padding,
                        item_pos.y + (item_size.y - option_size.y) * 0.5,
                    );
                    renderer.draw_text(option_pos, colors::TEXT_PRIMARY, option_text);
                }
            }
        }
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        match event {
            UiEvent::CursorMoved { position } => {
                self.main_hovered = self.main_area_contains(*position);
                if self.is_open {
                    self.update_hover_from_point(*position);
                }
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
                        if self.main_area_contains(*position) {
                            if self.is_open {
                                self.close();
                            } else {
                                self.open();
                                self.update_hover_from_point(*position);
                            }
                            None
                        } else if self.is_open {
                            if let Some(index) = self.index_at_point(*position) {
                                self.apply_selection(index)
                            } else {
                                if !self.list_area_contains(*position) {
                                    self.close();
                                }
                                None
                            }
                        } else {
                            None
                        }
                    }
                    ButtonState::Released => None,
                }
            }
            UiEvent::Scroll { delta, position } => {
                if !self.is_open {
                    return None;
                }
                if !self.list_area_contains(*position) {
                    return None;
                }
                if self.options.len() <= self.max_visible_items {
                    return None;
                }

                if *delta > 0.0 {
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                        self.update_hover_from_point(*position);
                    }
                } else if *delta < 0.0 {
                    let max_offset = self.max_scroll_offset();
                    if self.scroll_offset < max_offset {
                        self.scroll_offset += 1;
                        self.update_hover_from_point(*position);
                    }
                }
                None
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

    fn contains_point(&self, point: Vec2) -> bool {
        self.main_area_contains(point) || (self.is_open && self.list_area_contains(point))
    }
}

impl crate::ui::LayoutElement for Dropdown {
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
