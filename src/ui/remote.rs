use crate::colors::{self, PaletteSlot};
use crate::renderer::QuadRenderer;
use crate::ui::{
    Button, ButtonState, Checkbox, CrossAlignment, Dropdown, HorizontalLayout, Label, LayoutElement,
    Panel, TextBox, UiEvent, VerticalLayout, Widget, WidgetEvent,
};
use crate::Vec2;
use glam::Vec4;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use thiserror::Error;

/// Command transmitted over the remote control channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommand {
    /// Identifier of the UI element to target.
    pub id: String,
    /// Method to invoke on the target.
    pub method: String,
    /// JSON payload with method arguments. Optional depending on method.
    #[serde(default)]
    pub params: Value,
}

/// Thread-safe queue storing pending remote commands.
#[derive(Clone)]
pub struct RemoteCommandChannel {
    sender: mpsc::Sender<RemoteCommand>,
    receiver: Arc<Mutex<mpsc::Receiver<RemoteCommand>>>,
    pending: Arc<AtomicUsize>,
}

impl RemoteCommandChannel {
    /// Creates a new empty channel.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            pending: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Adds a parsed command to the queue.
    pub fn push(&self, command: RemoteCommand) {
        self.pending.fetch_add(1, Ordering::SeqCst);
        if self.sender.send(command).is_err() {
            self.pending.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Parses a JSON string into a command and enqueues it.
    pub fn push_json(&self, json: &str) -> Result<(), serde_json::Error> {
        let command: RemoteCommand = serde_json::from_str(json)?;
        self.push(command);
        Ok(())
    }

    /// Removes and returns all pending commands.
    pub fn drain(&self) -> Vec<RemoteCommand> {
        let mut drained = Vec::new();
        if let Ok(receiver) = self.receiver.lock() {
            loop {
                match receiver.try_recv() {
                    Ok(command) => drained.push(command),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }
        }
        let drained_count = drained.len();
        if drained_count > 0 {
            let _ = self
                .pending
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                    current.checked_sub(drained_count)
                });
        }
        drained
    }

    /// Returns current queue length. Intended for diagnostics and tests.
    pub fn len(&self) -> usize {
        self.pending.load(Ordering::SeqCst)
    }

    /// Returns true when no commands are pending.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Spawns a background thread that reads newline-delimited JSON commands from a reader.
    /// Each parsed command is pushed into the channel.
    pub fn spawn_reader_thread<R>(&self, reader: R) -> JoinHandle<Result<(), RemoteTransportError>>
    where
        R: Read + Send + 'static,
    {
        let channel = self.clone();
        thread::spawn(move || {
            let mut buf_reader = BufReader::new(reader);
            loop {
                let mut buffer = String::new();
                let bytes = buf_reader.read_line(&mut buffer)?;
                if bytes == 0 {
                    break;
                }
                match serde_json::from_str::<RemoteCommand>(buffer.trim()) {
                    Ok(command) => channel.push(command),
                    Err(err) => return Err(RemoteTransportError::Parse(err)),
                }
            }
            Ok(())
        })
    }

    /// Spawns a background worker that consumes JSON commands from a standard channel.
    ///
    /// Each received message must be a newline-delimited JSON string describing a [`RemoteCommand`].
    /// Empty messages are ignored.
    pub fn spawn_json_channel_listener(
        &self,
        receiver: mpsc::Receiver<String>,
    ) -> JoinHandle<Result<(), RemoteTransportError>> {
        let channel = self.clone();
        thread::spawn(move || {
            while let Ok(message) = receiver.recv() {
                let trimmed = message.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let command: RemoteCommand =
                    serde_json::from_str(trimmed).map_err(RemoteTransportError::Parse)?;
                channel.push(command);
            }
            Ok(())
        })
    }
}

impl Default for RemoteCommandChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors produced while transporting remote commands.
#[derive(Debug, Error)]
pub enum RemoteTransportError {
    #[error("failed to parse remote command: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

/// Outcome of processing the pending remote commands.
#[derive(Debug, Default)]
pub struct RemoteSessionReport {
    /// Number of commands successfully executed.
    pub processed: usize,
    /// Errors encountered while handling commands.
    pub errors: Vec<RemoteError>,
}

impl RemoteSessionReport {
    /// Returns true when the session executed every command without errors.
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// High-level API for connecting UI elements with the remote command queue.
///
/// Construct a session per update tick, register element references, run
/// [`RemoteUiSession::process`] to apply all queued commands, then drop the session.
pub struct RemoteUiSession<'a> {
    channel: Option<&'a RemoteCommandChannel>,
    targets: HashMap<String, Box<dyn RemoteTarget + 'a>>,
}

impl<'a> RemoteUiSession<'a> {
    /// Creates a new session bound to the shared command channel.
    pub fn new(channel: &'a RemoteCommandChannel) -> Self {
        Self {
            channel: Some(channel),
            targets: HashMap::new(),
        }
    }

    /// Creates a session that will be supplied commands manually.
    pub fn detached() -> Self {
        Self {
            channel: None,
            targets: HashMap::new(),
        }
    }

    /// Registers a button instance under the provided identifier.
    pub fn with_button(mut self, id: impl Into<String>, button: &'a mut Button) -> Self {
        self.targets
            .insert(id.into(), Box::new(ButtonTarget { button }));
        self
    }

    /// Registers a checkbox instance.
    pub fn with_checkbox(mut self, id: impl Into<String>, checkbox: &'a mut Checkbox) -> Self {
        self.targets
            .insert(id.into(), Box::new(CheckboxTarget { checkbox }));
        self
    }

    /// Registers a label instance.
    pub fn with_label(mut self, id: impl Into<String>, label: &'a mut Label) -> Self {
        self.targets
            .insert(id.into(), Box::new(LabelTarget { label }));
        self
    }

    /// Registers a text box instance.
    pub fn with_textbox(mut self, id: impl Into<String>, textbox: &'a mut TextBox) -> Self {
        self.targets
            .insert(id.into(), Box::new(TextBoxTarget { textbox }));
        self
    }

    /// Registers a dropdown instance.
    pub fn with_dropdown(mut self, id: impl Into<String>, dropdown: &'a mut Dropdown) -> Self {
        self.targets
            .insert(id.into(), Box::new(DropdownTarget { dropdown }));
        self
    }

    /// Registers a panel instance.
    pub fn with_panel(mut self, id: impl Into<String>, panel: &'a mut Panel) -> Self {
        self.targets
            .insert(id.into(), Box::new(PanelTarget { panel }));
        self
    }

    /// Registers a horizontal layout instance.
    pub fn with_horizontal_layout(
        mut self,
        id: impl Into<String>,
        layout: &'a mut HorizontalLayout,
    ) -> Self {
        self.targets
            .insert(id.into(), Box::new(HorizontalLayoutTarget { layout }));
        self
    }

    /// Registers a vertical layout instance.
    pub fn with_vertical_layout(
        mut self,
        id: impl Into<String>,
        layout: &'a mut VerticalLayout,
    ) -> Self {
        self.targets
            .insert(id.into(), Box::new(VerticalLayoutTarget { layout }));
        self
    }

    /// Processes all commands currently queued on the shared channel.
    ///
    /// Unregistered identifiers produce an [`RemoteError::UnknownTarget`] entry
    /// in the returned report.
    pub fn process(self) -> RemoteSessionReport {
        let commands = match self.channel {
            Some(channel) => channel.drain(),
            None => Vec::new(),
        };
        self.process_with_commands(commands)
    }

    /// Processes commands supplied by the caller.
    pub fn process_with_commands(mut self, commands: Vec<RemoteCommand>) -> RemoteSessionReport {
        let mut report = RemoteSessionReport::default();
        for command in commands {
            match self.targets.get_mut(&command.id) {
                Some(target) => match target.invoke(&command.method, &command.params) {
                    Ok(()) => report.processed += 1,
                    Err(error) => report
                        .errors
                        .push(error.into_remote_error(command.id.clone())),
                },
                None => report.errors.push(RemoteError::UnknownTarget {
                    id: command.id.clone(),
                    method: command.method.clone(),
                }),
            }
        }
        report
    }
}

/// Error surfaced while handling a remote command.
#[derive(Debug, Error)]
pub enum RemoteError {
    #[error("no widget registered with id '{id}' when invoking '{method}'")]
    UnknownTarget { id: String, method: String },
    #[error("widget '{id}' already exists when invoking '{method}'")]
    AlreadyExists { id: String, method: String },
    #[error("method '{method}' is not supported by {target}")]
    UnsupportedMethod {
        id: String,
        method: String,
        target: &'static str,
    },
    #[error("invalid parameters for '{method}' on {target}: {source}")]
    InvalidParams {
        id: String,
        method: String,
        target: &'static str,
        #[source]
        source: serde_json::Error,
    },
}

/// Owns widget instances that can be manipulated through remote commands.
pub struct RemoteUiHost {
    channel: RemoteCommandChannel,
    widgets: HashMap<String, HostedWidget>,
    draw_order: Vec<String>,
}

impl RemoteUiHost {
    /// Creates a host backed by the provided command channel.
    pub fn new(channel: RemoteCommandChannel) -> Self {
        Self {
            channel,
            widgets: HashMap::new(),
            draw_order: Vec::new(),
        }
    }

    /// Returns a clone of the underlying command channel for producers.
    pub fn command_channel(&self) -> RemoteCommandChannel {
        self.channel.clone()
    }

    /// Applies all queued commands, including create/destroy operations.
    pub fn process(&mut self) -> RemoteSessionReport {
        let commands = self.channel.drain();
        let mut report = RemoteSessionReport::default();
        for command in commands {
            match command.method.as_str() {
                "create" => match self.create_widget(&command) {
                    Ok(()) => report.processed += 1,
                    Err(err) => report.errors.push(err),
                },
                "destroy" => match self.destroy_widget(&command) {
                    Ok(()) => report.processed += 1,
                    Err(err) => report.errors.push(err),
                },
                "attach_child" => match self.attach_child(&command) {
                    Ok(()) => report.processed += 1,
                    Err(err) => report.errors.push(err),
                },
                "clear_all" => {
                    self.clear_all();
                    report.processed += 1;
                }
                "set_palette" => match self.set_palette(&command) {
                    Ok(()) => report.processed += 1,
                    Err(err) => report.errors.push(err),
                },
                "set_palette_slot" => match self.set_palette_slot(&command) {
                    Ok(()) => report.processed += 1,
                    Err(err) => report.errors.push(err),
                },
                _ => match self.apply_widget_command(&command) {
                    Ok(()) => report.processed += 1,
                    Err(err) => report.errors.push(err),
                },
            }
        }
        report
    }

    /// Draws all hosted widgets in creation order.
    pub fn draw(&self, renderer: &QuadRenderer) {
        for id in &self.draw_order {
            if let Some(widget) = self.widgets.get(id) {
                widget.draw(renderer);
            }
        }
        for id in &self.draw_order {
            if let Some(widget) = self.widgets.get(id) {
                widget.draw_overlay(renderer);
            }
        }
    }

    /// Dispatches an input event to hosted widgets, returning emitted events and a hit flag.
    pub fn handle_event(&mut self, event: &UiEvent) -> (Vec<WidgetEvent>, bool) {
        let mut events = Vec::new();
        let mut mouse_hit = false;
        let mouse_position = Self::event_position(event);
        let capture_pointer = matches!(
            event,
            UiEvent::MouseButton {
                state: ButtonState::Pressed,
                ..
            } | UiEvent::Scroll { .. }
        );

        let overlay_index = if capture_pointer {
            mouse_position.and_then(|position| {
                self.draw_order
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, id)| {
                        self.widgets.get(id).and_then(|widget| match widget {
                            HostedWidget::Dropdown(dropdown)
                                if dropdown.overlay_contains_point(position) =>
                            {
                                Some(index)
                            }
                            _ => None,
                        })
                    })
            })
        } else {
            None
        };

        if let Some(index) = overlay_index {
            if let Some(id) = self.draw_order.get(index) {
                if let Some(widget) = self.widgets.get_mut(id) {
                    mouse_hit = true;
                    if let Some(widget_event) = widget.handle_event(event) {
                        events.push(widget_event);
                    }
                }
            }
        }

        let mut pointer_claimed = capture_pointer && overlay_index.is_some();

        for index in (0..self.draw_order.len()).rev() {
            if Some(index) == overlay_index {
                continue;
            }

            let id = &self.draw_order[index];
            if let Some(widget) = self.widgets.get_mut(id) {
                let hit = mouse_position
                    .map_or(false, |position| widget.contains_point(position));

                if !mouse_hit && hit {
                    mouse_hit = true;
                }

                if capture_pointer && pointer_claimed && hit {
                    continue;
                }

                if let Some(widget_event) = widget.handle_event(event) {
                    events.push(widget_event);
                }

                if capture_pointer && hit {
                    pointer_claimed = true;
                }
            }
        }
        (events, mouse_hit)
    }

    /// Returns true when a widget with the given identifier is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.widgets.contains_key(id)
    }

    /// Returns true if any hosted widget currently has input focus.
    pub fn has_focused_widget(&self) -> bool {
        self.widgets
            .values()
            .any(|widget| self.widget_has_focus(widget))
    }

    fn widget_has_focus(&self, widget: &HostedWidget) -> bool {
        match widget {
            HostedWidget::TextBox(textbox) => textbox.is_focused(),
            HostedWidget::Panel(panel) => self.panel_has_focus(panel),
            HostedWidget::HorizontalLayout(layout) => self.horizontal_layout_has_focus(layout),
            HostedWidget::VerticalLayout(layout) => self.vertical_layout_has_focus(layout),
            HostedWidget::Attached(attached) => self.attached_widget_has_focus(attached),
            _ => false,
        }
    }

    fn panel_has_focus(&self, panel: &Panel) -> bool {
        (0..panel.len()).any(|index| {
            panel
                .child(index)
                .map_or(false, |child| self.layout_element_has_focus(child))
        })
    }

    fn horizontal_layout_has_focus(&self, layout: &HorizontalLayout) -> bool {
        (0..layout.len()).any(|index| {
            layout
                .child(index)
                .map_or(false, |child| self.layout_element_has_focus(child))
        })
    }

    fn vertical_layout_has_focus(&self, layout: &VerticalLayout) -> bool {
        (0..layout.len()).any(|index| {
            layout
                .child(index)
                .map_or(false, |child| self.layout_element_has_focus(child))
        })
    }

    fn layout_element_has_focus(&self, element: &dyn LayoutElement) -> bool {
        if let Some(textbox) = element.as_any().downcast_ref::<TextBox>() {
            textbox.is_focused()
        } else if let Some(panel) = element.as_any().downcast_ref::<Panel>() {
            self.panel_has_focus(panel)
        } else if let Some(layout) = element.as_any().downcast_ref::<HorizontalLayout>() {
            self.horizontal_layout_has_focus(layout)
        } else if let Some(layout) = element.as_any().downcast_ref::<VerticalLayout>() {
            self.vertical_layout_has_focus(layout)
        } else {
            false
        }
    }

    fn attached_widget_has_focus(&self, attached: &AttachedWidget) -> bool {
        self.resolve_attached_element(attached)
            .map_or(false, |element| self.layout_element_has_focus(element))
    }

    fn resolve_attached_element<'a>(
        &'a self,
        attached: &'a AttachedWidget,
    ) -> Option<&'a dyn LayoutElement> {
        let parent_id = attached.parent.id();

        if let Some(HostedWidget::Attached(parent_attached)) = self.widgets.get(parent_id) {
            let parent_element = self.resolve_attached_element(parent_attached)?;
            return Self::child_from_parent_element(
                parent_element,
                &attached.parent,
                attached.slot,
            );
        }

        match (&attached.parent, self.widgets.get(parent_id)?) {
            (ParentLink::Panel(_), HostedWidget::Panel(panel)) => panel.child(attached.slot),
            (ParentLink::HorizontalLayout(_), HostedWidget::HorizontalLayout(layout)) => {
                layout.child(attached.slot)
            }
            (ParentLink::VerticalLayout(_), HostedWidget::VerticalLayout(layout)) => {
                layout.child(attached.slot)
            }
            _ => None,
        }
    }

    fn child_from_parent_element<'a>(
        parent_element: &'a dyn LayoutElement,
        parent_link: &ParentLink,
        slot: usize,
    ) -> Option<&'a dyn LayoutElement> {
        match parent_link {
            ParentLink::Panel(_) => parent_element
                .as_any()
                .downcast_ref::<Panel>()
                .and_then(|panel| panel.child(slot)),
            ParentLink::HorizontalLayout(_) => parent_element
                .as_any()
                .downcast_ref::<HorizontalLayout>()
                .and_then(|layout| layout.child(slot)),
            ParentLink::VerticalLayout(_) => parent_element
                .as_any()
                .downcast_ref::<VerticalLayout>()
                .and_then(|layout| layout.child(slot)),
        }
    }

    fn event_position(event: &UiEvent) -> Option<Vec2> {
        match event {
            UiEvent::CursorMoved { position }
            | UiEvent::MouseButton { position, .. }
            | UiEvent::Scroll { position, .. } => Some(*position),
            _ => None,
        }
    }

    /// Number of hosted widgets.
    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    /// Returns true if the child is attached to the given parent.
    pub fn is_attached_to(&self, child_id: &str, parent_id: &str) -> bool {
        matches!(
            self.widgets.get(child_id),
            Some(HostedWidget::Attached(attached)) if attached.parent.id() == parent_id
        )
    }

    fn apply_widget_command(&mut self, command: &RemoteCommand) -> Result<(), RemoteError> {
        let mut widget = match self.widgets.remove(&command.id) {
            Some(entry) => entry,
            None => {
                return Err(RemoteError::UnknownTarget {
                    id: command.id.clone(),
                    method: command.method.clone(),
                })
            }
        };

        let result = widget.invoke(self, &command.id, &command.method, &command.params);

        self.widgets.insert(command.id.clone(), widget);
        result
    }

    fn create_widget(&mut self, command: &RemoteCommand) -> Result<(), RemoteError> {
        if self.widgets.contains_key(&command.id) {
            return Err(RemoteError::AlreadyExists {
                id: command.id.clone(),
                method: command.method.clone(),
            });
        }

        let payload: CreateWidgetPayload =
            serde_json::from_value(command.params.clone()).map_err(|err| {
                RemoteError::InvalidParams {
                    id: command.id.clone(),
                    method: command.method.clone(),
                    target: "RemoteUiHost",
                    source: err,
                }
            })?;

        let kind = payload.kind.to_ascii_lowercase();
        match kind.as_str() {
            "button" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let size = payload
                    .size
                    .clone()
                    .unwrap_or_else(|| SizePayload {
                        width: 120.0,
                        height: 36.0,
                    })
                    .into_vec2();
                let label = payload
                    .label
                    .clone()
                    .or_else(|| payload.text.clone())
                    .unwrap_or_else(|| "Button".to_string());
                let button = Button::new(command.id.clone(), position, size, label);
                self.insert_widget(command.id.clone(), HostedWidget::Button(button));
            }
            "checkbox" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let size = payload
                    .size
                    .clone()
                    .unwrap_or_else(|| SizePayload {
                        width: 24.0,
                        height: 24.0,
                    })
                    .into_vec2();
                let label = payload
                    .label
                    .clone()
                    .unwrap_or_else(|| "Checkbox".to_string());
                let mut checkbox = Checkbox::new(command.id.clone(), position, size, label);
                if let Some(checked) = payload.checked {
                    checkbox.set_checked(checked);
                }
                self.insert_widget(command.id.clone(), HostedWidget::Checkbox(checkbox));
            }
            "label" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let size = payload
                    .size
                    .clone()
                    .unwrap_or_else(|| SizePayload {
                        width: 160.0,
                        height: 28.0,
                    })
                    .into_vec2();
                let text = payload.text.clone().unwrap_or_else(|| "Label".to_string());
                let label_widget = match payload.color.clone() {
                    Some(color_payload) => Label::new(
                        command.id.clone(),
                        position,
                        size,
                        text.clone(),
                        color_payload.into_vec4(),
                    ),
                    None => Label::with_palette_color(
                        command.id.clone(),
                        position,
                        size,
                        text,
                        PaletteSlot::AccentSoft,
                    ),
                };
                self.insert_widget(command.id.clone(), HostedWidget::Label(label_widget));
            }
            "textbox" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let size = payload
                    .size
                    .clone()
                    .unwrap_or_else(|| SizePayload {
                        width: 200.0,
                        height: 32.0,
                    })
                    .into_vec2();
                let placeholder = payload
                    .placeholder
                    .clone()
                    .unwrap_or_else(|| "Type here...".to_string());
                let mut textbox = TextBox::new(command.id.clone(), position, size, placeholder);
                if let Some(value) = payload.text.clone() {
                    textbox.set_text(value);
                }
                self.insert_widget(command.id.clone(), HostedWidget::TextBox(textbox));
            }
            "dropdown" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let size = payload
                    .size
                    .clone()
                    .unwrap_or_else(|| SizePayload {
                        width: 180.0,
                        height: 34.0,
                    })
                    .into_vec2();
                let opts = payload.options.clone().unwrap_or_default();
                let mut dropdown = Dropdown::new(position, size, command.id.clone(), opts);
                if let Some(placeholder) = payload.placeholder.clone() {
                    dropdown = dropdown.with_placeholder(placeholder);
                }
                if let Some(count) = payload.max_visible_items {
                    dropdown.set_max_visible_items(count);
                }
                if let Some(height) = payload.option_height {
                    dropdown.set_option_height(height);
                }
                if let Some(index) = payload.selected_index {
                    dropdown.set_selected_index(index);
                }
                self.insert_widget(command.id.clone(), HostedWidget::Dropdown(dropdown));
            }
            "panel" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let size = payload
                    .size
                    .clone()
                    .unwrap_or_else(|| SizePayload {
                        width: 280.0,
                        height: 200.0,
                    })
                    .into_vec2();
                let title = payload.title.clone().unwrap_or_else(|| "Panel".to_string());
                let panel = Panel::new(command.id.clone(), position, size, title);
                self.insert_widget(command.id.clone(), HostedWidget::Panel(panel));
            }
            "horizontal_layout" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let layout = HorizontalLayout::new(command.id.clone(), position);
                self.insert_widget(command.id.clone(), HostedWidget::HorizontalLayout(layout));
            }
            "vertical_layout" => {
                let position = payload
                    .position
                    .clone()
                    .unwrap_or_else(|| PositionPayload { x: 0.0, y: 0.0 })
                    .into_vec2();
                let layout = VerticalLayout::new(command.id.clone(), position);
                self.insert_widget(command.id.clone(), HostedWidget::VerticalLayout(layout));
            }
            _ => {
                return Err(RemoteError::UnsupportedMethod {
                    id: command.id.clone(),
                    method: format!("{}::{}", command.method, kind),
                    target: "RemoteUiHost",
                });
            }
        }

        Ok(())
    }

    fn attach_child(&mut self, command: &RemoteCommand) -> Result<(), RemoteError> {
        let payload: AttachChildPayload =
            serde_json::from_value(command.params.clone()).map_err(|err| {
                RemoteError::InvalidParams {
                    id: command.id.clone(),
                    method: command.method.clone(),
                    target: "RemoteUiHost",
                    source: err,
                }
            })?;

        let parent_id = command.id.clone();
        let child_id = payload.child.clone();
        let offset = payload.offset.map(|p| p.into_vec2());

        let child_entry = match self.widgets.remove(&child_id) {
            Some(entry) => entry,
            None => {
                return Err(RemoteError::UnknownTarget {
                    id: child_id,
                    method: command.method.clone(),
                })
            }
        };

        if let Some(attached_meta) = self.widgets.get(&parent_id).and_then(|widget| {
            if let HostedWidget::Attached(attached) = widget {
                Some(attached.clone())
            } else {
                None
            }
        }) {
            let result = attached_meta.attach_child(
                self,
                &parent_id,
                &command.method,
                &child_id,
                child_entry,
                offset,
            );
            return match result {
                Ok((parent_link, kind, slot)) => {
                    self.draw_order.retain(|id| id != &child_id);
                    self.widgets.insert(
                        child_id.clone(),
                        HostedWidget::Attached(AttachedWidget {
                            parent: parent_link,
                            slot,
                            kind,
                        }),
                    );
                    Ok(())
                }
                Err((err, original)) => {
                    self.widgets.insert(child_id, original);
                    Err(err)
                }
            };
        }

        let parent_entry = match self.widgets.get_mut(&parent_id) {
            Some(entry) => entry,
            None => {
                self.widgets.insert(child_id.clone(), child_entry);
                return Err(RemoteError::UnknownTarget {
                    id: parent_id,
                    method: command.method.clone(),
                });
            }
        };

        let result = match parent_entry {
            HostedWidget::Panel(panel) => {
                let offset = offset.unwrap_or(Vec2::ZERO);
                attach_into_panel(panel, child_entry, offset, &child_id, &command.method)
                    .map(|(kind, slot)| (ParentLink::Panel(parent_id.clone()), kind, slot))
            }
            HostedWidget::HorizontalLayout(layout) => {
                attach_into_horizontal(layout, child_entry, &child_id, &command.method).map(
                    |(kind, slot)| (ParentLink::HorizontalLayout(parent_id.clone()), kind, slot),
                )
            }
            HostedWidget::VerticalLayout(layout) => {
                attach_into_vertical(layout, child_entry, &child_id, &command.method)
                    .map(|(kind, slot)| (ParentLink::VerticalLayout(parent_id.clone()), kind, slot))
            }
            other => Err((
                RemoteError::UnsupportedMethod {
                    id: parent_id.clone(),
                    method: command.method.clone(),
                    target: other.type_name(),
                },
                child_entry,
            )),
        };

        match result {
            Ok((parent_link, kind, slot)) => {
                self.draw_order.retain(|id| id != &child_id);
                self.widgets.insert(
                    child_id.clone(),
                    HostedWidget::Attached(AttachedWidget {
                        parent: parent_link,
                        slot,
                        kind,
                    }),
                );
                Ok(())
            }
            Err((err, original)) => {
                self.widgets.insert(child_id, original);
                Err(err)
            }
        }
    }

    fn destroy_widget(&mut self, command: &RemoteCommand) -> Result<(), RemoteError> {
        match self.widgets.remove(&command.id) {
            Some(HostedWidget::Attached(attached)) => {
                self.detach_from_parent(attached)?;
                Ok(())
            }
            Some(_) => {
                self.draw_order.retain(|current| current != &command.id);
                let parent_id = command.id.clone();
                let to_remove: Vec<String> = self
                    .widgets
                    .iter()
                    .filter_map(|(child_id, widget)| {
                        if let HostedWidget::Attached(attached) = widget {
                            if attached.parent.id() == parent_id {
                                return Some(child_id.clone());
                            }
                        }
                        None
                    })
                    .collect();
                for child_id in to_remove {
                    self.widgets.remove(&child_id);
                }
                Ok(())
            }
            None => Err(RemoteError::UnknownTarget {
                id: command.id.clone(),
                method: command.method.clone(),
            }),
        }
    }

    fn clear_all(&mut self) {
        self.widgets.clear();
        self.draw_order.clear();
    }

    fn set_palette(&mut self, command: &RemoteCommand) -> Result<(), RemoteError> {
        let payload: SetPalettePayload = Self::parse_host_params(command)?;
        let mut palette = colors::palette();
        let changed = payload.apply(&mut palette);
        if changed {
            colors::set_palette(palette);
        }
        Ok(())
    }

    fn set_palette_slot(&mut self, command: &RemoteCommand) -> Result<(), RemoteError> {
        let payload: SetPaletteSlotPayload = Self::parse_host_params(command)?;
        colors::set_palette_slot(payload.slot, payload.color.into_vec4());
        Ok(())
    }

    fn parse_host_params<T>(command: &RemoteCommand) -> Result<T, RemoteError>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(command.params.clone()).map_err(|err| RemoteError::InvalidParams {
            id: command.id.clone(),
            method: command.method.clone(),
            target: "RemoteUiHost",
            source: err,
        })
    }

    fn insert_widget(&mut self, id: String, widget: HostedWidget) {
        self.draw_order.push(id.clone());
        self.widgets.insert(id, widget);
    }

    fn detach_from_parent(&mut self, attachment: AttachedWidget) -> Result<(), RemoteError> {
        let parent_id = attachment.parent.id().to_string();
        let method = "destroy".to_string();
        match (&attachment.parent, self.widgets.get_mut(&parent_id)) {
            (ParentLink::Panel(_), Some(HostedWidget::Panel(panel))) => {
                if panel.remove_child(attachment.slot).is_some() {
                    self.adjust_child_indices(&parent_id, attachment.slot);
                    Ok(())
                } else {
                    Err(RemoteError::UnsupportedMethod {
                        id: parent_id,
                        method,
                        target: attachment.kind.target_name(),
                    })
                }
            }
            (ParentLink::HorizontalLayout(_), Some(HostedWidget::HorizontalLayout(layout))) => {
                if layout.remove_child(attachment.slot).is_some() {
                    self.adjust_child_indices(&parent_id, attachment.slot);
                    Ok(())
                } else {
                    Err(RemoteError::UnsupportedMethod {
                        id: parent_id,
                        method,
                        target: attachment.kind.target_name(),
                    })
                }
            }
            (ParentLink::VerticalLayout(_), Some(HostedWidget::VerticalLayout(layout))) => {
                if layout.remove_child(attachment.slot).is_some() {
                    self.adjust_child_indices(&parent_id, attachment.slot);
                    Ok(())
                } else {
                    Err(RemoteError::UnsupportedMethod {
                        id: parent_id,
                        method,
                        target: attachment.kind.target_name(),
                    })
                }
            }
            (_, Some(parent)) => Err(RemoteError::UnsupportedMethod {
                id: parent_id,
                method,
                target: parent.type_name(),
            }),
            (_, None) => Err(RemoteError::UnknownTarget {
                id: parent_id,
                method,
            }),
        }
    }

    fn adjust_child_indices(&mut self, parent_id: &str, removed_slot: usize) {
        for widget in self.widgets.values_mut() {
            if let HostedWidget::Attached(attached) = widget {
                if attached.parent.id() == parent_id && attached.slot > removed_slot {
                    attached.slot -= 1;
                }
            }
        }
    }
}

enum HostedWidget {
    Button(Button),
    Checkbox(Checkbox),
    Label(Label),
    TextBox(TextBox),
    Dropdown(Dropdown),
    Panel(Panel),
    HorizontalLayout(HorizontalLayout),
    VerticalLayout(VerticalLayout),
    Attached(AttachedWidget),
}

impl HostedWidget {
    fn type_name(&self) -> &'static str {
        match self {
            HostedWidget::Button(widget) => widget.type_name(),
            HostedWidget::Checkbox(widget) => widget.type_name(),
            HostedWidget::Label(widget) => widget.type_name(),
            HostedWidget::TextBox(widget) => widget.type_name(),
            HostedWidget::Dropdown(widget) => widget.type_name(),
            HostedWidget::Panel(widget) => widget.type_name(),
            HostedWidget::HorizontalLayout(widget) => widget.type_name(),
            HostedWidget::VerticalLayout(widget) => widget.type_name(),
            HostedWidget::Attached(attached) => attached.kind.target_name(),
        }
    }

    fn draw(&self, renderer: &QuadRenderer) {
        match self {
            HostedWidget::Button(widget) => widget.draw(renderer),
            HostedWidget::Checkbox(widget) => widget.draw(renderer),
            HostedWidget::Label(widget) => widget.draw(renderer),
            HostedWidget::TextBox(widget) => widget.draw(renderer),
            HostedWidget::Dropdown(widget) => widget.draw(renderer),
            HostedWidget::Panel(widget) => widget.draw(renderer),
            HostedWidget::HorizontalLayout(widget) => widget.draw(renderer),
            HostedWidget::VerticalLayout(widget) => widget.draw(renderer),
            HostedWidget::Attached(_) => {}
        }
    }

    fn draw_overlay(&self, renderer: &QuadRenderer) {
        match self {
            HostedWidget::Button(widget) => widget.draw_overlay(renderer),
            HostedWidget::Checkbox(widget) => widget.draw_overlay(renderer),
            HostedWidget::Label(widget) => widget.draw_overlay(renderer),
            HostedWidget::TextBox(widget) => widget.draw_overlay(renderer),
            HostedWidget::Dropdown(widget) => widget.draw_overlay(renderer),
            HostedWidget::Panel(widget) => widget.draw_overlay(renderer),
            HostedWidget::HorizontalLayout(widget) => widget.draw_overlay(renderer),
            HostedWidget::VerticalLayout(widget) => widget.draw_overlay(renderer),
            HostedWidget::Attached(_) => {}
        }
    }

    fn handle_event(&mut self, event: &UiEvent) -> Option<WidgetEvent> {
        match self {
            HostedWidget::Button(widget) => widget.handle_event(event),
            HostedWidget::Checkbox(widget) => widget.handle_event(event),
            HostedWidget::Label(_) => None,
            HostedWidget::TextBox(widget) => widget.handle_event(event),
            HostedWidget::Dropdown(widget) => widget.handle_event(event),
            HostedWidget::Panel(widget) => widget.handle_event(event),
            HostedWidget::HorizontalLayout(widget) => widget.handle_event(event),
            HostedWidget::VerticalLayout(widget) => widget.handle_event(event),
            HostedWidget::Attached(_) => None,
        }
    }

    fn contains_point(&self, point: Vec2) -> bool {
        match self {
            HostedWidget::Button(widget) => widget.contains_point(point),
            HostedWidget::Checkbox(widget) => widget.contains_point(point),
            HostedWidget::Label(widget) => widget.contains_point(point),
            HostedWidget::TextBox(widget) => widget.contains_point(point),
            HostedWidget::Dropdown(widget) => widget.contains_point(point),
            HostedWidget::Panel(widget) => widget.contains_point(point),
            HostedWidget::HorizontalLayout(widget) => widget.contains_point(point),
            HostedWidget::VerticalLayout(widget) => widget.contains_point(point),
            HostedWidget::Attached(_) => false,
        }
    }

    #[allow(dead_code)]
    fn register<'a>(&'a mut self, session: RemoteUiSession<'a>, id: &str) -> RemoteUiSession<'a> {
        match self {
            HostedWidget::Button(widget) => session.with_button(id.to_string(), widget),
            HostedWidget::Checkbox(widget) => session.with_checkbox(id.to_string(), widget),
            HostedWidget::Label(widget) => session.with_label(id.to_string(), widget),
            HostedWidget::TextBox(widget) => session.with_textbox(id.to_string(), widget),
            HostedWidget::Dropdown(widget) => session.with_dropdown(id.to_string(), widget),
            HostedWidget::Panel(widget) => session.with_panel(id.to_string(), widget),
            HostedWidget::HorizontalLayout(widget) => {
                session.with_horizontal_layout(id.to_string(), widget)
            }
            HostedWidget::VerticalLayout(widget) => {
                session.with_vertical_layout(id.to_string(), widget)
            }
            HostedWidget::Attached(_) => session,
        }
    }

    fn invoke(
        &mut self,
        host: &mut RemoteUiHost,
        id: &str,
        method: &str,
        params: &Value,
    ) -> Result<(), RemoteError> {
        match self {
            HostedWidget::Button(widget) => {
                invoke_target(ButtonTarget { button: widget }, id, method, params)
            }
            HostedWidget::Checkbox(widget) => {
                invoke_target(CheckboxTarget { checkbox: widget }, id, method, params)
            }
            HostedWidget::Label(widget) => {
                invoke_target(LabelTarget { label: widget }, id, method, params)
            }
            HostedWidget::TextBox(widget) => {
                invoke_target(TextBoxTarget { textbox: widget }, id, method, params)
            }
            HostedWidget::Dropdown(widget) => {
                invoke_target(DropdownTarget { dropdown: widget }, id, method, params)
            }
            HostedWidget::Panel(widget) => {
                invoke_target(PanelTarget { panel: widget }, id, method, params)
            }
            HostedWidget::HorizontalLayout(widget) => invoke_target(
                HorizontalLayoutTarget { layout: widget },
                id,
                method,
                params,
            ),
            HostedWidget::VerticalLayout(widget) => {
                invoke_target(VerticalLayoutTarget { layout: widget }, id, method, params)
            }
            HostedWidget::Attached(attached) => attached.invoke(host, id, method, params),
        }
    }
}

fn invoke_target<T: RemoteTarget>(
    mut target: T,
    id: &str,
    method: &str,
    params: &Value,
) -> Result<(), RemoteError> {
    target
        .invoke(method, params)
        .map_err(|err| err.into_remote_error(id.to_string()))
}

#[derive(Clone)]
struct AttachedWidget {
    parent: ParentLink,
    slot: usize,
    kind: AttachedKind,
}

#[derive(Clone)]
enum ParentLink {
    Panel(String),
    HorizontalLayout(String),
    VerticalLayout(String),
}

impl ParentLink {
    fn id(&self) -> &str {
        match self {
            ParentLink::Panel(id)
            | ParentLink::HorizontalLayout(id)
            | ParentLink::VerticalLayout(id) => id,
        }
    }
}

#[derive(Clone, Copy)]
enum AttachedKind {
    Button,
    Checkbox,
    Label,
    TextBox,
    Dropdown,
    Panel,
    HorizontalLayout,
    VerticalLayout,
}

impl AttachedKind {
    fn target_name(&self) -> &'static str {
        match self {
            AttachedKind::Button => "Button",
            AttachedKind::Checkbox => "Checkbox",
            AttachedKind::Label => "Label",
            AttachedKind::TextBox => "TextBox",
            AttachedKind::Dropdown => "Dropdown",
            AttachedKind::Panel => "Panel",
            AttachedKind::HorizontalLayout => "HorizontalLayout",
            AttachedKind::VerticalLayout => "VerticalLayout",
        }
    }
}

impl AttachedWidget {
    fn invoke(
        &self,
        host: &mut RemoteUiHost,
        id: &str,
        method: &str,
        params: &Value,
    ) -> Result<(), RemoteError> {
        match self.kind {
            AttachedKind::Button => {
                let button = self.borrow_child_mut::<Button>(host, id, method)?;
                invoke_target(ButtonTarget { button }, id, method, params)
            }
            AttachedKind::Checkbox => {
                let checkbox = self.borrow_child_mut::<Checkbox>(host, id, method)?;
                invoke_target(CheckboxTarget { checkbox }, id, method, params)
            }
            AttachedKind::Label => {
                let label = self.borrow_child_mut::<Label>(host, id, method)?;
                invoke_target(LabelTarget { label }, id, method, params)
            }
            AttachedKind::TextBox => {
                let textbox = self.borrow_child_mut::<TextBox>(host, id, method)?;
                invoke_target(TextBoxTarget { textbox }, id, method, params)
            }
            AttachedKind::Dropdown => {
                let dropdown = self.borrow_child_mut::<Dropdown>(host, id, method)?;
                invoke_target(DropdownTarget { dropdown }, id, method, params)
            }
            AttachedKind::Panel => {
                let panel = self.borrow_child_mut::<Panel>(host, id, method)?;
                invoke_target(PanelTarget { panel }, id, method, params)
            }
            AttachedKind::HorizontalLayout => {
                let layout = self.borrow_child_mut::<HorizontalLayout>(host, id, method)?;
                invoke_target(HorizontalLayoutTarget { layout }, id, method, params)
            }
            AttachedKind::VerticalLayout => {
                let layout = self.borrow_child_mut::<VerticalLayout>(host, id, method)?;
                invoke_target(VerticalLayoutTarget { layout }, id, method, params)
            }
        }
    }

    fn borrow_child_mut<'a, T: LayoutElement + 'static>(
        &self,
        host: &'a mut RemoteUiHost,
        request_id: &str,
        method: &str,
    ) -> Result<&'a mut T, RemoteError> {
        fn downcast_child<'a, T: LayoutElement + 'static>(
            child: Option<&'a mut dyn LayoutElement>,
            request_id: &str,
            method: &str,
            kind: AttachedKind,
        ) -> Result<&'a mut T, RemoteError> {
            child
                .and_then(|child| child.as_any_mut().downcast_mut::<T>())
                .ok_or_else(|| RemoteError::UnsupportedMethod {
                    id: request_id.to_string(),
                    method: method.to_string(),
                    target: kind.target_name(),
                })
        }

        let parent_key = self.parent.id();

        if let Some(attached_meta) = host.widgets.get(parent_key).and_then(|widget| {
            if let HostedWidget::Attached(attached) = widget {
                Some(attached.clone())
            } else {
                None
            }
        }) {
            let attached_kind = attached_meta.kind;
            match (&self.parent, attached_kind) {
                (ParentLink::Panel(_), AttachedKind::Panel) => {
                    let panel =
                        attached_meta.borrow_child_mut::<Panel>(host, request_id, method)?;
                    return downcast_child(
                        panel.child_mut(self.slot),
                        request_id,
                        method,
                        self.kind,
                    );
                }
                (ParentLink::HorizontalLayout(_), AttachedKind::HorizontalLayout) => {
                    let layout = attached_meta
                        .borrow_child_mut::<HorizontalLayout>(host, request_id, method)?;
                    return downcast_child(
                        layout.child_mut(self.slot),
                        request_id,
                        method,
                        self.kind,
                    );
                }
                (ParentLink::VerticalLayout(_), AttachedKind::VerticalLayout) => {
                    let layout = attached_meta
                        .borrow_child_mut::<VerticalLayout>(host, request_id, method)?;
                    return downcast_child(
                        layout.child_mut(self.slot),
                        request_id,
                        method,
                        self.kind,
                    );
                }
                (_, other_kind) => {
                    return Err(RemoteError::UnsupportedMethod {
                        id: parent_key.to_string(),
                        method: method.to_string(),
                        target: other_kind.target_name(),
                    });
                }
            }
        }

        let existing_type = host
            .widgets
            .get(parent_key)
            .map(|widget| widget.type_name());

        match self.parent {
            ParentLink::Panel(_) => {
                if let Some(HostedWidget::Panel(panel)) = host.widgets.get_mut(parent_key) {
                    return downcast_child(
                        panel.child_mut(self.slot),
                        request_id,
                        method,
                        self.kind,
                    );
                }
            }
            ParentLink::HorizontalLayout(_) => {
                if let Some(HostedWidget::HorizontalLayout(layout)) =
                    host.widgets.get_mut(parent_key)
                {
                    return downcast_child(
                        layout.child_mut(self.slot),
                        request_id,
                        method,
                        self.kind,
                    );
                }
            }
            ParentLink::VerticalLayout(_) => {
                if let Some(HostedWidget::VerticalLayout(layout)) = host.widgets.get_mut(parent_key)
                {
                    return downcast_child(
                        layout.child_mut(self.slot),
                        request_id,
                        method,
                        self.kind,
                    );
                }
            }
        }

        if let Some(target) = existing_type {
            return Err(RemoteError::UnsupportedMethod {
                id: parent_key.to_string(),
                method: method.to_string(),
                target,
            });
        }

        Err(RemoteError::UnknownTarget {
            id: parent_key.to_string(),
            method: method.to_string(),
        })
    }

    fn attach_child(
        &self,
        host: &mut RemoteUiHost,
        parent_id: &str,
        method: &str,
        child_id: &str,
        child: HostedWidget,
        offset: Option<Vec2>,
    ) -> Result<(ParentLink, AttachedKind, usize), (RemoteError, HostedWidget)> {
        match self.kind {
            AttachedKind::Panel => {
                let panel = match self.borrow_child_mut::<Panel>(host, parent_id, method) {
                    Ok(panel) => panel,
                    Err(err) => return Err((err, child)),
                };
                let offset = offset.unwrap_or(Vec2::ZERO);
                attach_into_panel(panel, child, offset, child_id, method)
                    .map(|(kind, slot)| (ParentLink::Panel(parent_id.to_string()), kind, slot))
            }
            AttachedKind::HorizontalLayout => {
                let layout =
                    match self.borrow_child_mut::<HorizontalLayout>(host, parent_id, method) {
                        Ok(layout) => layout,
                        Err(err) => return Err((err, child)),
                    };
                attach_into_horizontal(layout, child, child_id, method).map(|(kind, slot)| {
                    (
                        ParentLink::HorizontalLayout(parent_id.to_string()),
                        kind,
                        slot,
                    )
                })
            }
            AttachedKind::VerticalLayout => {
                let layout = match self.borrow_child_mut::<VerticalLayout>(host, parent_id, method)
                {
                    Ok(layout) => layout,
                    Err(err) => return Err((err, child)),
                };
                attach_into_vertical(layout, child, child_id, method).map(|(kind, slot)| {
                    (
                        ParentLink::VerticalLayout(parent_id.to_string()),
                        kind,
                        slot,
                    )
                })
            }
            _ => Err((
                RemoteError::UnsupportedMethod {
                    id: parent_id.to_string(),
                    method: method.to_string(),
                    target: self.kind.target_name(),
                },
                child,
            )),
        }
    }
}

fn attach_into_panel(
    panel: &mut Panel,
    child: HostedWidget,
    offset: Vec2,
    child_id: &str,
    method: &str,
) -> Result<(AttachedKind, usize), (RemoteError, HostedWidget)> {
    match child {
        HostedWidget::Button(button) => {
            panel.add_child(button, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::Button, slot))
        }
        HostedWidget::Checkbox(checkbox) => {
            panel.add_child(checkbox, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::Checkbox, slot))
        }
        HostedWidget::Label(label) => {
            panel.add_child(label, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::Label, slot))
        }
        HostedWidget::TextBox(textbox) => {
            panel.add_child(textbox, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::TextBox, slot))
        }
        HostedWidget::Dropdown(dropdown) => {
            panel.add_child(dropdown, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::Dropdown, slot))
        }
        HostedWidget::Panel(inner_panel) => {
            panel.add_child(inner_panel, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::Panel, slot))
        }
        HostedWidget::HorizontalLayout(layout) => {
            panel.add_child(layout, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::HorizontalLayout, slot))
        }
        HostedWidget::VerticalLayout(layout) => {
            panel.add_child(layout, offset);
            let slot = panel.len().saturating_sub(1);
            Ok((AttachedKind::VerticalLayout, slot))
        }
        HostedWidget::Attached(attached) => Err((
            RemoteError::UnsupportedMethod {
                id: child_id.to_string(),
                method: method.to_string(),
                target: attached.kind.target_name(),
            },
            HostedWidget::Attached(attached),
        )),
    }
}

fn attach_into_horizontal(
    layout: &mut HorizontalLayout,
    child: HostedWidget,
    child_id: &str,
    method: &str,
) -> Result<(AttachedKind, usize), (RemoteError, HostedWidget)> {
    match child {
        HostedWidget::Button(button) => {
            layout.add_child(button);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Button, slot))
        }
        HostedWidget::Checkbox(checkbox) => {
            layout.add_child(checkbox);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Checkbox, slot))
        }
        HostedWidget::Label(label) => {
            layout.add_child(label);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Label, slot))
        }
        HostedWidget::TextBox(textbox) => {
            layout.add_child(textbox);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::TextBox, slot))
        }
        HostedWidget::Dropdown(dropdown) => {
            layout.add_child(dropdown);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Dropdown, slot))
        }
        HostedWidget::Panel(panel) => {
            layout.add_child(panel);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Panel, slot))
        }
        HostedWidget::HorizontalLayout(inner) => {
            layout.add_child(inner);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::HorizontalLayout, slot))
        }
        HostedWidget::VerticalLayout(inner) => {
            layout.add_child(inner);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::VerticalLayout, slot))
        }
        HostedWidget::Attached(attached) => Err((
            RemoteError::UnsupportedMethod {
                id: child_id.to_string(),
                method: method.to_string(),
                target: attached.kind.target_name(),
            },
            HostedWidget::Attached(attached),
        )),
    }
}

fn attach_into_vertical(
    layout: &mut VerticalLayout,
    child: HostedWidget,
    child_id: &str,
    method: &str,
) -> Result<(AttachedKind, usize), (RemoteError, HostedWidget)> {
    match child {
        HostedWidget::Button(button) => {
            layout.add_child(button);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Button, slot))
        }
        HostedWidget::Checkbox(checkbox) => {
            layout.add_child(checkbox);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Checkbox, slot))
        }
        HostedWidget::Label(label) => {
            layout.add_child(label);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Label, slot))
        }
        HostedWidget::TextBox(textbox) => {
            layout.add_child(textbox);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::TextBox, slot))
        }
        HostedWidget::Dropdown(dropdown) => {
            layout.add_child(dropdown);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Dropdown, slot))
        }
        HostedWidget::Panel(panel) => {
            layout.add_child(panel);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::Panel, slot))
        }
        HostedWidget::HorizontalLayout(inner) => {
            layout.add_child(inner);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::HorizontalLayout, slot))
        }
        HostedWidget::VerticalLayout(inner) => {
            layout.add_child(inner);
            let slot = layout.len().saturating_sub(1);
            Ok((AttachedKind::VerticalLayout, slot))
        }
        HostedWidget::Attached(attached) => Err((
            RemoteError::UnsupportedMethod {
                id: child_id.to_string(),
                method: method.to_string(),
                target: attached.kind.target_name(),
            },
            HostedWidget::Attached(attached),
        )),
    }
}
trait RemoteTarget {
    fn target_name(&self) -> &'static str;
    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError>;
}

struct ButtonTarget<'a> {
    button: &'a mut Button,
}

struct CheckboxTarget<'a> {
    checkbox: &'a mut Checkbox,
}

struct LabelTarget<'a> {
    label: &'a mut Label,
}

struct TextBoxTarget<'a> {
    textbox: &'a mut TextBox,
}

struct DropdownTarget<'a> {
    dropdown: &'a mut Dropdown,
}

struct PanelTarget<'a> {
    panel: &'a mut Panel,
}

struct HorizontalLayoutTarget<'a> {
    layout: &'a mut HorizontalLayout,
}

struct VerticalLayoutTarget<'a> {
    layout: &'a mut VerticalLayout,
}

impl<'a> RemoteTarget for ButtonTarget<'a> {
    fn target_name(&self) -> &'static str {
        "Button"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.button.set_position(payload.into_vec2());
            }
            "set_size" => {
                let payload: SizePayload = parse_params(method, target, params)?;
                self.button.set_size(payload.into_vec2());
            }
            "set_label" => {
                let payload: TextPayload = parse_params(method, target, params)?;
                self.button.set_label(payload.text);
            }
            "set_colors" => {
                let payload: ButtonColorsPayload = parse_params(method, target, params)?;
                self.button.set_colors(
                    payload.normal.into_vec4(),
                    payload.hover.into_vec4(),
                    payload.pressed.into_vec4(),
                );
            }
            "set_text_color" => {
                let payload: ColorPayload = parse_params(method, target, params)?;
                self.button.set_text_color(payload.into_vec4());
            }
            "set_border_color" => {
                let payload: ColorPayload = parse_params(method, target, params)?;
                self.button.set_border_color(payload.into_vec4());
            }
            "set_hovered" => {
                let payload: BoolPayload = parse_params(method, target, params)?;
                self.button.set_hovered(payload.value);
            }
            "set_pressed" => {
                let payload: BoolPayload = parse_params(method, target, params)?;
                self.button.set_pressed(payload.value);
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

impl<'a> RemoteTarget for CheckboxTarget<'a> {
    fn target_name(&self) -> &'static str {
        "Checkbox"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.checkbox.set_position(payload.into_vec2());
            }
            "set_size" => {
                let payload: SizePayload = parse_params(method, target, params)?;
                self.checkbox.set_size(payload.into_vec2());
            }
            "set_checked" => {
                let payload: BoolPayload = parse_params(method, target, params)?;
                self.checkbox.set_checked(payload.value);
            }
            "set_label" => {
                let payload: TextPayload = parse_params(method, target, params)?;
                self.checkbox.set_label(payload.text);
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

impl<'a> RemoteTarget for LabelTarget<'a> {
    fn target_name(&self) -> &'static str {
        "Label"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.label.set_position(payload.into_vec2());
            }
            "set_size" => {
                let payload: SizePayload = parse_params(method, target, params)?;
                self.label.set_size(payload.into_vec2());
            }
            "set_text" => {
                let payload: TextPayload = parse_params(method, target, params)?;
                log::debug!("Setting label text: {}", payload.text);
                self.label.set_text(payload.text);
            }
            "set_color" => {
                let payload: ColorPayload = parse_params(method, target, params)?;
                self.label.set_color(payload.into_vec4());
            }
            "set_palette_color" => {
                let payload: PaletteSlotPayload = parse_params(method, target, params)?;
                self.label.set_palette_color(payload.slot);
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

impl<'a> RemoteTarget for TextBoxTarget<'a> {
    fn target_name(&self) -> &'static str {
        "TextBox"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.textbox.set_position(payload.into_vec2());
            }
            "set_size" => {
                let payload: SizePayload = parse_params(method, target, params)?;
                self.textbox.set_size(payload.into_vec2());
            }
            "set_text" => {
                let payload: TextPayload = parse_params(method, target, params)?;
                self.textbox.set_text(payload.text);
            }
            "set_focused" => {
                let payload: BoolPayload = parse_params(method, target, params)?;
                self.textbox.set_focused(payload.value);
            }
            "set_placeholder" => {
                let payload: TextPayload = parse_params(method, target, params)?;
                self.textbox.set_placeholder(payload.text);
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

impl<'a> RemoteTarget for DropdownTarget<'a> {
    fn target_name(&self) -> &'static str {
        "Dropdown"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.dropdown.set_position(payload.into_vec2());
            }
            "set_size" => {
                let payload: SizePayload = parse_params(method, target, params)?;
                self.dropdown.set_size(payload.into_vec2());
            }
            "set_selected_index" => {
                let payload: IndexPayload = parse_params(method, target, params)?;
                self.dropdown.set_selected_index(payload.index);
            }
            "set_options" => {
                let payload: OptionsPayload = parse_params(method, target, params)?;
                self.dropdown.set_options(payload.options);
            }
            "set_placeholder" => {
                let payload: OptionalTextPayload = parse_params(method, target, params)?;
                self.dropdown.set_placeholder(payload.text);
            }
            "set_max_visible_items" => {
                let payload: CountPayload = parse_params(method, target, params)?;
                self.dropdown.set_max_visible_items(payload.count);
            }
            "set_option_height" => {
                let payload: FloatPayload = parse_params(method, target, params)?;
                self.dropdown.set_option_height(payload.value);
            }
            "set_open" => {
                let payload: BoolPayload = parse_params(method, target, params)?;
                self.dropdown.set_open(payload.value);
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

impl<'a> RemoteTarget for PanelTarget<'a> {
    fn target_name(&self) -> &'static str {
        "Panel"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.panel.set_position(payload.into_vec2());
            }
            "set_size" => {
                let payload: SizePayload = parse_params(method, target, params)?;
                self.panel.set_size(payload.into_vec2());
            }
            "set_title" => {
                let payload: TextPayload = parse_params(method, target, params)?;
                self.panel.set_title(payload.text);
            }
            "set_colors" => {
                let payload: PanelColorsPayload = parse_params(method, target, params)?;
                self.panel.set_colors(
                    payload.background.into_vec4(),
                    payload.title_bar.into_vec4(),
                );
            }
            "set_border_color" => {
                let payload: ColorPayload = parse_params(method, target, params)?;
                self.panel.set_border_color(payload.into_vec4());
            }
            "set_padding" => {
                let payload: PaddingPayload = parse_params(method, target, params)?;
                self.panel.set_padding(payload.into_vec2());
            }
            "clear_children" => {
                self.panel.clear_children();
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

impl<'a> RemoteTarget for HorizontalLayoutTarget<'a> {
    fn target_name(&self) -> &'static str {
        "HorizontalLayout"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.layout.set_position(payload.into_vec2());
            }
            "set_spacing" => {
                let payload: FloatPayload = parse_params(method, target, params)?;
                self.layout.set_spacing(payload.value);
            }
            "set_padding" => {
                let payload: PaddingPayload = parse_params(method, target, params)?;
                self.layout.set_padding(payload.into_vec2());
            }
            "set_cross_alignment" => {
                let payload: AlignmentPayload = parse_params(method, target, params)?;
                let alignment = payload
                    .into_alignment()
                    .ok_or_else(|| TargetError::unsupported(method, target))?;
                self.layout.set_cross_alignment(alignment);
            }
            "recompute_layout" => {
                self.layout.recompute_layout();
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

impl<'a> RemoteTarget for VerticalLayoutTarget<'a> {
    fn target_name(&self) -> &'static str {
        "VerticalLayout"
    }

    fn invoke(&mut self, method: &str, params: &Value) -> Result<(), TargetError> {
        let target = self.target_name();
        match method {
            "set_position" => {
                let payload: PositionPayload = parse_params(method, target, params)?;
                self.layout.set_position(payload.into_vec2());
            }
            "set_spacing" => {
                let payload: FloatPayload = parse_params(method, target, params)?;
                self.layout.set_spacing(payload.value);
            }
            "set_padding" => {
                let payload: PaddingPayload = parse_params(method, target, params)?;
                self.layout.set_padding(payload.into_vec2());
            }
            "set_cross_alignment" => {
                let payload: AlignmentPayload = parse_params(method, target, params)?;
                let alignment = payload
                    .into_alignment()
                    .ok_or_else(|| TargetError::unsupported(method, target))?;
                self.layout.set_cross_alignment(alignment);
            }
            "recompute_layout" => {
                self.layout.recompute_layout();
            }
            other => return Err(TargetError::unsupported(other, target)),
        }
        Ok(())
    }
}

#[derive(Debug)]
enum TargetError {
    Unsupported {
        method: String,
        target: &'static str,
    },
    InvalidParams {
        method: String,
        target: &'static str,
        source: serde_json::Error,
    },
}

impl TargetError {
    fn unsupported(method: &str, target: &'static str) -> Self {
        TargetError::Unsupported {
            method: method.to_string(),
            target,
        }
    }

    fn invalid(method: &str, target: &'static str, source: serde_json::Error) -> Self {
        TargetError::InvalidParams {
            method: method.to_string(),
            target,
            source,
        }
    }

    fn into_remote_error(self, id: String) -> RemoteError {
        match self {
            TargetError::Unsupported { method, target } => {
                RemoteError::UnsupportedMethod { id, method, target }
            }
            TargetError::InvalidParams {
                method,
                target,
                source,
            } => RemoteError::InvalidParams {
                id,
                method,
                target,
                source,
            },
        }
    }
}

fn parse_params<T>(method: &str, target: &'static str, params: &Value) -> Result<T, TargetError>
where
    T: DeserializeOwned,
{
    serde_json::from_value(params.clone()).map_err(|err| TargetError::invalid(method, target, err))
}

#[derive(Clone, Deserialize)]
struct PositionPayload {
    x: f32,
    y: f32,
}

impl PositionPayload {
    fn into_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[derive(Clone, Deserialize)]
struct SizePayload {
    width: f32,
    height: f32,
}

impl SizePayload {
    fn into_vec2(self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

#[derive(Clone, Deserialize)]
struct PaddingPayload {
    x: f32,
    y: f32,
}

impl PaddingPayload {
    fn into_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[derive(Clone, Deserialize)]
struct TextPayload {
    text: String,
}

#[derive(Clone, Deserialize)]
struct OptionalTextPayload {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Clone, Deserialize)]
struct BoolPayload {
    value: bool,
}

#[derive(Clone, Deserialize)]
struct FloatPayload {
    value: f32,
}

#[derive(Clone, Deserialize)]
struct CountPayload {
    count: usize,
}

#[derive(Clone, Deserialize)]
struct IndexPayload {
    index: usize,
}

#[derive(Clone, Deserialize)]
struct OptionsPayload {
    options: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct AlignmentPayload {
    alignment: String,
}

impl AlignmentPayload {
    fn into_alignment(self) -> Option<CrossAlignment> {
        match self.alignment.to_ascii_lowercase().as_str() {
            "start" => Some(CrossAlignment::Start),
            "center" | "centre" => Some(CrossAlignment::Center),
            "end" => Some(CrossAlignment::End),
            _ => None,
        }
    }
}

#[derive(Clone, Deserialize)]
struct ColorPayload {
    r: f32,
    g: f32,
    b: f32,
    #[serde(default = "default_alpha")]
    a: f32,
}

impl ColorPayload {
    fn into_vec4(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }
}

#[derive(Clone, Deserialize)]
struct ButtonColorsPayload {
    normal: ColorPayload,
    hover: ColorPayload,
    pressed: ColorPayload,
}

#[derive(Clone, Deserialize)]
struct PanelColorsPayload {
    background: ColorPayload,
    title_bar: ColorPayload,
}

#[derive(Clone, Deserialize)]
struct AttachChildPayload {
    child: String,
    #[serde(default)]
    offset: Option<PositionPayload>,
}

#[derive(Clone, Deserialize)]
struct CreateWidgetPayload {
    kind: String,
    #[serde(default)]
    position: Option<PositionPayload>,
    #[serde(default)]
    size: Option<SizePayload>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    color: Option<ColorPayload>,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(default)]
    options: Option<Vec<String>>,
    #[serde(default)]
    checked: Option<bool>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    max_visible_items: Option<usize>,
    #[serde(default)]
    option_height: Option<f32>,
    #[serde(default)]
    selected_index: Option<usize>,
}

#[derive(Clone, Deserialize)]
struct PaletteSlotPayload {
    slot: PaletteSlot,
}

#[derive(Clone, Deserialize)]
struct SetPalettePayload {
    #[serde(default)]
    text_primary: Option<ColorPayload>,
    #[serde(default)]
    text_secondary: Option<ColorPayload>,
    #[serde(default)]
    surface_dark: Option<ColorPayload>,
    #[serde(default)]
    surface: Option<ColorPayload>,
    #[serde(default)]
    surface_light: Option<ColorPayload>,
    #[serde(default)]
    accent: Option<ColorPayload>,
    #[serde(default)]
    accent_soft: Option<ColorPayload>,
    #[serde(default)]
    border_soft: Option<ColorPayload>,
    #[serde(default)]
    border_subtle: Option<ColorPayload>,
    #[serde(default)]
    checkmark: Option<ColorPayload>,
    #[serde(default)]
    shadow: Option<ColorPayload>,
}

impl SetPalettePayload {
    fn apply(self, palette: &mut colors::Palette) -> bool {
        let SetPalettePayload {
            text_primary,
            text_secondary,
            surface_dark,
            surface,
            surface_light,
            accent,
            accent_soft,
            border_soft,
            border_subtle,
            checkmark,
            shadow,
        } = self;

        let mut changed = false;

        if let Some(color) = text_primary {
            palette.text_primary = color.into_vec4();
            changed = true;
        }
        if let Some(color) = text_secondary {
            palette.text_secondary = color.into_vec4();
            changed = true;
        }
        if let Some(color) = surface_dark {
            palette.surface_dark = color.into_vec4();
            changed = true;
        }
        if let Some(color) = surface {
            palette.surface = color.into_vec4();
            changed = true;
        }
        if let Some(color) = surface_light {
            palette.surface_light = color.into_vec4();
            changed = true;
        }
        if let Some(color) = accent {
            palette.accent = color.into_vec4();
            changed = true;
        }
        if let Some(color) = accent_soft {
            palette.accent_soft = color.into_vec4();
            changed = true;
        }
        if let Some(color) = border_soft {
            palette.border_soft = color.into_vec4();
            changed = true;
        }
        if let Some(color) = border_subtle {
            palette.border_subtle = color.into_vec4();
            changed = true;
        }
        if let Some(color) = checkmark {
            palette.checkmark = color.into_vec4();
            changed = true;
        }
        if let Some(color) = shadow {
            palette.shadow = color.into_vec4();
            changed = true;
        }

        changed
    }
}

#[derive(Clone, Deserialize)]
struct SetPaletteSlotPayload {
    slot: PaletteSlot,
    color: ColorPayload,
}

fn default_alpha() -> f32 {
    1.0
}
