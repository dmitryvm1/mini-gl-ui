use crate::renderer::QuadRenderer;
use crate::ui::{ButtonState, Dropdown, UiEvent, Widget, WidgetEvent};
use glam::Vec2;
use std::any::Any;
use std::slice::{Iter, IterMut};

/// Alignment applied on the axis perpendicular to a linear layout's direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossAlignment {
    Start,
    Center,
    End,
}

/// Trait implemented by widgets that can participate in layout containers.
pub trait LayoutElement: Widget + Any {
    /// Updates the widget position.
    fn set_position(&mut self, position: Vec2);

    /// Type-erased immutable access for downcasting support.
    fn as_any(&self) -> &dyn Any;

    /// Type-erased mutable access for downcasting support.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

fn align_offset(max_dimension: f32, child_dimension: f32, alignment: CrossAlignment) -> f32 {
    match alignment {
        CrossAlignment::Start => 0.0,
        CrossAlignment::Center => (max_dimension - child_dimension) * 0.5,
        CrossAlignment::End => max_dimension - child_dimension,
    }
}

/// Iterator over immutable layout children.
pub struct LayoutChildren<'a> {
    inner: Iter<'a, Box<dyn LayoutElement>>,
}

impl<'a> Iterator for LayoutChildren<'a> {
    type Item = &'a dyn LayoutElement;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|child| child.as_ref())
    }
}

/// Iterator over mutable layout children.
pub struct LayoutChildrenMut<'a> {
    inner: IterMut<'a, Box<dyn LayoutElement>>,
}

impl<'a> Iterator for LayoutChildrenMut<'a> {
    type Item = &'a mut dyn LayoutElement;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|child| child.as_mut())
    }
}

/// A layout that arranges children horizontally.
pub struct HorizontalLayout {
    id: String,
    position: Vec2,
    size: Vec2,
    spacing: f32,
    padding: Vec2,
    cross_alignment: CrossAlignment,
    children: Vec<Box<dyn LayoutElement>>,
}

impl HorizontalLayout {
    /// Creates a new horizontal layout at the given position.
    pub fn new(id: impl Into<String>, position: Vec2) -> Self {
        let mut layout = Self {
            id: id.into(),
            position,
            size: Vec2::ZERO,
            spacing: 12.0,
            padding: Vec2::splat(8.0),
            cross_alignment: CrossAlignment::Start,
            children: Vec::new(),
        };
        layout.recompute_layout();
        layout
    }

    /// Sets the spacing between child widgets (builder-style).
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self.recompute_layout();
        self
    }

    /// Sets the padding applied symmetrically on both axes (builder-style).
    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = Vec2::new(padding.x.max(0.0), padding.y.max(0.0));
        self.recompute_layout();
        self
    }

    /// Sets the cross-axis alignment for children (builder-style).
    pub fn with_cross_alignment(mut self, alignment: CrossAlignment) -> Self {
        self.cross_alignment = alignment;
        self.recompute_layout();
        self
    }

    /// Updates the layout position.
    pub fn set_position(&mut self, position: Vec2) {
        if self.position != position {
            self.position = position;
            self.recompute_layout();
        }
    }

    /// Updates spacing between child widgets.
    pub fn set_spacing(&mut self, spacing: f32) {
        let spacing = spacing.max(0.0);
        if (self.spacing - spacing).abs() > f32::EPSILON {
            self.spacing = spacing;
            self.recompute_layout();
        }
    }

    /// Updates layout padding.
    pub fn set_padding(&mut self, padding: Vec2) {
        let padding = Vec2::new(padding.x.max(0.0), padding.y.max(0.0));
        if self.padding != padding {
            self.padding = padding;
            self.recompute_layout();
        }
    }

    /// Updates the cross-axis alignment.
    pub fn set_cross_alignment(&mut self, alignment: CrossAlignment) {
        if self.cross_alignment != alignment {
            self.cross_alignment = alignment;
            self.recompute_layout();
        }
    }

    /// Adds a child widget to the layout.
    pub fn add_child<W>(&mut self, child: W)
    where
        W: LayoutElement + 'static,
    {
        self.children.push(Box::new(child));
        self.recompute_layout();
    }

    /// Inserts a child widget at the specified index.
    pub fn insert_child<W>(&mut self, index: usize, child: W)
    where
        W: LayoutElement + 'static,
    {
        let index = index.min(self.children.len());
        self.children.insert(index, Box::new(child));
        self.recompute_layout();
    }

    /// Removes and returns the child widget at `index`.
    pub fn remove_child(&mut self, index: usize) -> Option<Box<dyn LayoutElement>> {
        let removed = if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        };
        if removed.is_some() {
            self.recompute_layout();
        }
        removed
    }

    /// Clears all child widgets.
    pub fn clear(&mut self) {
        if !self.children.is_empty() {
            self.children.clear();
            self.recompute_layout();
        }
    }

    /// Returns the number of child widgets.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns true if the layout has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Provides immutable access to a child widget.
    pub fn child(&self, index: usize) -> Option<&dyn LayoutElement> {
        self.children.get(index).map(|child| child.as_ref())
    }

    /// Provides mutable access to a child widget.
    pub fn child_mut(&mut self, index: usize) -> Option<&mut dyn LayoutElement> {
        self.children.get_mut(index).map(|child| child.as_mut())
    }

    /// Returns an iterator over immutable child widgets.
    pub fn children(&self) -> LayoutChildren<'_> {
        LayoutChildren {
            inner: self.children.iter(),
        }
    }

    /// Returns an iterator over mutable child widgets.
    pub fn children_mut(&mut self) -> LayoutChildrenMut<'_> {
        LayoutChildrenMut {
            inner: self.children.iter_mut(),
        }
    }

    /// Recomputes child positions. Call after mutating a child via `child_mut`.
    pub fn recompute_layout(&mut self) {
        let origin = self.position + Vec2::new(self.padding.x, self.padding.y);

        if self.children.is_empty() {
            self.size = Vec2::new(self.padding.x * 2.0, self.padding.y * 2.0);
            return;
        }

        let max_height = self
            .children
            .iter()
            .map(|child| child.size().y)
            .fold(0.0, f32::max);

        let mut cursor = origin.x;
        let mut total_width = self.padding.x * 2.0;
        let spacing = self.spacing;
        let last_index = self.children.len().saturating_sub(1);

        for (index, child) in self.children.iter_mut().enumerate() {
            let child_size = child.size();
            let y = origin.y + align_offset(max_height, child_size.y, self.cross_alignment);
            child.set_position(Vec2::new(cursor, y));
            cursor += child_size.x;
            total_width += child_size.x;
            if index < last_index {
                cursor += spacing;
                total_width += spacing;
            }
        }

        let total_height = max_height + self.padding.y * 2.0;
        self.size = Vec2::new(
            total_width.max(self.padding.x * 2.0),
            total_height.max(self.padding.y * 2.0),
        );
    }
}

impl Widget for HorizontalLayout {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, renderer: &QuadRenderer) {
        for child in &self.children {
            child.draw(renderer);
        }
    }

    fn draw_overlay(&self, renderer: &QuadRenderer) {
        for child in &self.children {
            child.draw_overlay(renderer);
        }
    }

    fn type_name(&self) -> &'static str {
        "HorizontalLayout"
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
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
                if let Some(widget_event) = child.handle_event(event) {
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
                .map_or(false, |position| child.contains_point(position));

            if capture_pointer && pointer_claimed && hit {
                continue;
            }

            if let Some(widget_event) = child.handle_event(event) {
                return Some(widget_event);
            }

            if capture_pointer && hit {
                pointer_claimed = true;
            }
        }
        None
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn contains_point(&self, point: Vec2) -> bool {
        for child in &self.children {
            if child.contains_point(point) {
                return true;
            }
        }

        point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y
    }
}

impl LayoutElement for HorizontalLayout {
    fn set_position(&mut self, position: Vec2) {
        HorizontalLayout::set_position(self, position);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A layout that arranges children vertically.
pub struct VerticalLayout {
    id: String,
    position: Vec2,
    size: Vec2,
    spacing: f32,
    padding: Vec2,
    cross_alignment: CrossAlignment,
    children: Vec<Box<dyn LayoutElement>>,
}

impl VerticalLayout {
    /// Creates a new vertical layout at the given position.
    pub fn new(id: impl Into<String>, position: Vec2) -> Self {
        let mut layout = Self {
            id: id.into(),
            position,
            size: Vec2::ZERO,
            spacing: 12.0,
            padding: Vec2::splat(8.0),
            cross_alignment: CrossAlignment::Start,
            children: Vec::new(),
        };
        layout.recompute_layout();
        layout
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self.recompute_layout();
        self
    }

    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = Vec2::new(padding.x.max(0.0), padding.y.max(0.0));
        self.recompute_layout();
        self
    }

    pub fn with_cross_alignment(mut self, alignment: CrossAlignment) -> Self {
        self.cross_alignment = alignment;
        self.recompute_layout();
        self
    }

    pub fn set_position(&mut self, position: Vec2) {
        if self.position != position {
            self.position = position;
            self.recompute_layout();
        }
    }

    pub fn set_spacing(&mut self, spacing: f32) {
        let spacing = spacing.max(0.0);
        if (self.spacing - spacing).abs() > f32::EPSILON {
            self.spacing = spacing;
            self.recompute_layout();
        }
    }

    pub fn set_padding(&mut self, padding: Vec2) {
        let padding = Vec2::new(padding.x.max(0.0), padding.y.max(0.0));
        if self.padding != padding {
            self.padding = padding;
            self.recompute_layout();
        }
    }

    pub fn set_cross_alignment(&mut self, alignment: CrossAlignment) {
        if self.cross_alignment != alignment {
            self.cross_alignment = alignment;
            self.recompute_layout();
        }
    }

    pub fn add_child<W>(&mut self, child: W)
    where
        W: LayoutElement + 'static,
    {
        self.children.push(Box::new(child));
        self.recompute_layout();
    }

    pub fn insert_child<W>(&mut self, index: usize, child: W)
    where
        W: LayoutElement + 'static,
    {
        let index = index.min(self.children.len());
        self.children.insert(index, Box::new(child));
        self.recompute_layout();
    }

    pub fn remove_child(&mut self, index: usize) -> Option<Box<dyn LayoutElement>> {
        let removed = if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        };
        if removed.is_some() {
            self.recompute_layout();
        }
        removed
    }

    pub fn clear(&mut self) {
        if !self.children.is_empty() {
            self.children.clear();
            self.recompute_layout();
        }
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn child(&self, index: usize) -> Option<&dyn LayoutElement> {
        self.children.get(index).map(|child| child.as_ref())
    }

    pub fn child_mut(&mut self, index: usize) -> Option<&mut dyn LayoutElement> {
        self.children.get_mut(index).map(|child| child.as_mut())
    }

    pub fn children(&self) -> LayoutChildren<'_> {
        LayoutChildren {
            inner: self.children.iter(),
        }
    }

    pub fn children_mut(&mut self) -> LayoutChildrenMut<'_> {
        LayoutChildrenMut {
            inner: self.children.iter_mut(),
        }
    }

    pub fn recompute_layout(&mut self) {
        let origin = self.position + Vec2::new(self.padding.x, self.padding.y);

        if self.children.is_empty() {
            self.size = Vec2::new(self.padding.x * 2.0, self.padding.y * 2.0);
            return;
        }

        let max_width = self
            .children
            .iter()
            .map(|child| child.size().x)
            .fold(0.0, f32::max);

        let mut cursor = origin.y;
        let mut total_height = self.padding.y * 2.0;
        let spacing = self.spacing;
        let last_index = self.children.len().saturating_sub(1);

        for (index, child) in self.children.iter_mut().enumerate() {
            let child_size = child.size();
            let x = origin.x + align_offset(max_width, child_size.x, self.cross_alignment);
            child.set_position(Vec2::new(x, cursor));
            cursor += child_size.y;
            total_height += child_size.y;
            if index < last_index {
                cursor += spacing;
                total_height += spacing;
            }
        }

        let total_width = max_width + self.padding.x * 2.0;
        self.size = Vec2::new(
            total_width.max(self.padding.x * 2.0),
            total_height.max(self.padding.y * 2.0),
        );
    }
}

impl Widget for VerticalLayout {
    fn id(&self) -> &str {
        &self.id
    }

    fn draw(&self, renderer: &QuadRenderer) {
        for child in &self.children {
            child.draw(renderer);
        }
    }

    fn draw_overlay(&self, renderer: &QuadRenderer) {
        for child in &self.children {
            child.draw_overlay(renderer);
        }
    }

    fn type_name(&self) -> &'static str {
        "VerticalLayout"
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
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
                if let Some(widget_event) = child.handle_event(event) {
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
                .map_or(false, |position| child.contains_point(position));

            if capture_pointer && pointer_claimed && hit {
                continue;
            }

            if let Some(widget_event) = child.handle_event(event) {
                return Some(widget_event);
            }

            if capture_pointer && hit {
                pointer_claimed = true;
            }
        }
        None
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn size(&self) -> Vec2 {
        self.size
    }

    fn contains_point(&self, point: Vec2) -> bool {
        for child in &self.children {
            if child.contains_point(point) {
                return true;
            }
        }

        point.x >= self.position.x
            && point.x <= self.position.x + self.size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.y
    }
}

impl LayoutElement for VerticalLayout {
    fn set_position(&mut self, position: Vec2) {
        VerticalLayout::set_position(self, position);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
