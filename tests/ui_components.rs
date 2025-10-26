use mini_gl_ui::{colors, ui::*, Vec2};
use serde_json::json;

#[test]
fn test_button_creation() {
    let button = Button::new(
        Vec2::new(10.0, 20.0),
        Vec2::new(100.0, 40.0),
        "Test".to_string(),
    );

    assert_eq!(button.position(), Vec2::new(10.0, 20.0));
    assert_eq!(button.size(), Vec2::new(100.0, 40.0));
    assert_eq!(button.label(), "Test");
    assert!(!button.is_pressed());
}

#[test]
fn test_button_state() {
    let mut button = Button::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 40.0),
        "Test".to_string(),
    );

    button.set_pressed(true);
    assert!(button.is_pressed());

    button.set_pressed(false);
    assert!(!button.is_pressed());
}

#[test]
fn test_checkbox_toggle() {
    let mut checkbox = Checkbox::new(
        Vec2::new(10.0, 10.0),
        Vec2::new(20.0, 20.0),
        "Option".to_string(),
    );

    assert!(!checkbox.is_checked());

    checkbox.toggle();
    assert!(checkbox.is_checked());

    checkbox.toggle();
    assert!(!checkbox.is_checked());
}

#[test]
fn test_textbox_text_input() {
    let mut textbox = TextBox::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0, 30.0),
        "Placeholder".to_string(),
    );

    assert_eq!(textbox.text(), "");

    textbox.insert_char('H');
    textbox.insert_char('i');
    assert_eq!(textbox.text(), "Hi");

    textbox.backspace();
    assert_eq!(textbox.text(), "H");
}

#[test]
fn test_panel_dragging() {
    let mut panel = Panel::new(
        Vec2::new(100.0, 100.0),
        Vec2::new(300.0, 200.0),
        "Panel".to_string(),
    );

    assert!(!panel.is_dragging());

    panel.start_drag(Vec2::new(150.0, 110.0));
    assert!(panel.is_dragging());

    panel.update_drag(Vec2::new(200.0, 160.0));
    assert_eq!(panel.position(), Vec2::new(150.0, 150.0));

    panel.stop_drag();
    assert!(!panel.is_dragging());
}

#[test]
fn test_label_creation() {
    let mut label = Label::new(
        Vec2::new(5.0, 5.0),
        Vec2::new(100.0, 20.0),
        "Label".to_string(),
        colors::BLUE,
    );

    assert_eq!(label.text(), "Label");

    label.set_text("New Text".to_string());
    assert_eq!(label.text(), "New Text");
}

#[test]
fn test_widget_contains_point() {
    let button = Button::new(
        Vec2::new(10.0, 10.0),
        Vec2::new(100.0, 50.0),
        "Test".to_string(),
    );

    // Point inside
    assert!(button.contains_point(Vec2::new(50.0, 30.0)));

    // Point on edge
    assert!(button.contains_point(Vec2::new(10.0, 10.0)));
    assert!(button.contains_point(Vec2::new(110.0, 60.0)));

    // Point outside
    assert!(!button.contains_point(Vec2::new(5.0, 5.0)));
    assert!(!button.contains_point(Vec2::new(120.0, 70.0)));
}

#[test]
fn test_panel_title_bar_hit_detection() {
    let panel = Panel::new(
        Vec2::new(100.0, 100.0),
        Vec2::new(200.0, 150.0),
        "Panel".to_string(),
    );

    // Point in title bar
    assert!(panel.title_bar_contains_point(Vec2::new(150.0, 110.0)));

    // Point in panel content (not title bar)
    assert!(!panel.title_bar_contains_point(Vec2::new(150.0, 150.0)));

    // Point outside panel
    assert!(!panel.title_bar_contains_point(Vec2::new(50.0, 50.0)));
}

#[test]
fn test_button_handle_event_emits_click() {
    let mut button = Button::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 40.0),
        "Click".to_string(),
    );

    button.handle_event(&UiEvent::CursorMoved {
        position: Vec2::new(50.0, 20.0),
    });
    button.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(50.0, 20.0),
    });
    let release_event = button.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Released,
        position: Vec2::new(50.0, 20.0),
    });

    match release_event {
        Some(WidgetEvent::ButtonClicked { label }) => assert_eq!(label, "Click"),
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn test_checkbox_handle_event_toggles() {
    let mut checkbox = Checkbox::new(
        Vec2::new(10.0, 10.0),
        Vec2::new(20.0, 20.0),
        "Option".to_string(),
    );

    let event = checkbox.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(15.0, 15.0),
    });
    assert!(checkbox.is_checked());
    match event {
        Some(WidgetEvent::CheckboxToggled { label, checked }) => {
            assert_eq!(label, "Option");
            assert!(checked);
        }
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn test_textbox_handle_event_updates_text() {
    let mut textbox = TextBox::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0, 30.0),
        "Placeholder".to_string(),
    );

    let focus_event = textbox.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(10.0, 10.0),
    });
    assert!(textbox.is_focused());
    assert!(matches!(
        focus_event,
        Some(WidgetEvent::TextBoxFocusChanged { focused: true })
    ));

    let text_event = textbox.handle_event(&UiEvent::CharacterInput('A'));
    assert_eq!(textbox.text(), "A");
    match text_event {
        Some(WidgetEvent::TextChanged { text }) => assert_eq!(text, "A"),
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn test_textbox_repeated_backspace_events() {
    let mut textbox = TextBox::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0, 30.0),
        "Placeholder".to_string(),
    );

    textbox.set_focused(true);
    textbox.handle_event(&UiEvent::CharacterInput('A'));
    textbox.handle_event(&UiEvent::CharacterInput('B'));
    assert_eq!(textbox.text(), "AB");

    textbox.handle_event(&UiEvent::KeyInput {
        key: KeyCode::Backspace,
    });
    assert_eq!(textbox.text(), "A");

    textbox.handle_event(&UiEvent::KeyInput {
        key: KeyCode::Backspace,
    });
    assert_eq!(textbox.text(), "");
}

#[test]
fn test_dropdown_selects_option() {
    let mut dropdown = Dropdown::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(160.0, 32.0),
        "dropdown".to_string(),
        vec!["One".to_string(), "Two".to_string(), "Three".to_string()],
    )
    .with_placeholder("Pick one".to_string());

    assert!(dropdown.selected().is_none());
    assert!(!dropdown.is_open());

    dropdown.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(10.0, 10.0),
    });
    assert!(dropdown.is_open());

    let selection_event = dropdown.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(10.0, dropdown.size().y + 28.0 + 10.0),
    });
    assert!(!dropdown.is_open());
    assert_eq!(dropdown.selected(), Some("Two"));

    match selection_event {
        Some(WidgetEvent::DropdownSelectionChanged { id, selected }) => {
            assert_eq!(id, "dropdown");
            assert_eq!(selected, "Two");
        }
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn test_dropdown_scroll_and_select_lower_option() {
    let mut dropdown = Dropdown::new(
        Vec2::new(0.0, 0.0),
        Vec2::new(160.0, 32.0),
        "dropdown".to_string(),
        (1..=6).map(|i| format!("Option {i}")).collect(),
    )
    .with_placeholder("Pick one".to_string())
    .with_max_visible_items(3);

    dropdown.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(10.0, 10.0),
    });
    assert!(dropdown.is_open());

    let list_point = Vec2::new(10.0, dropdown.size().y + 5.0);
    dropdown.handle_event(&UiEvent::Scroll {
        delta: -1.0,
        position: list_point,
    });
    dropdown.handle_event(&UiEvent::Scroll {
        delta: -1.0,
        position: list_point,
    });

    let selection_event = dropdown.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(10.0, dropdown.size().y + 2.0 * 28.0 + 14.0),
    });
    assert_eq!(dropdown.selected(), Some("Option 5"));

    match selection_event {
        Some(WidgetEvent::DropdownSelectionChanged { selected, .. }) => {
            assert_eq!(selected, "Option 5");
        }
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn test_panel_handle_event_drag_flow() {
    let mut panel = Panel::new(
        Vec2::new(100.0, 100.0),
        Vec2::new(200.0, 150.0),
        "Panel".to_string(),
    );

    let start_event = panel.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(150.0, 110.0),
    });
    assert!(matches!(start_event, Some(WidgetEvent::PanelDragStarted)));

    let drag_event = panel.handle_event(&UiEvent::CursorMoved {
        position: Vec2::new(200.0, 160.0),
    });
    assert!(matches!(
        drag_event,
        Some(WidgetEvent::PanelDragged { position })
            if position == panel.position()
    ));

    let stop_event = panel.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Released,
        position: Vec2::new(200.0, 160.0),
    });
    assert!(matches!(stop_event, Some(WidgetEvent::PanelDragEnded)));
}

#[test]
fn panel_child_receives_events() {
    let mut panel = Panel::new(
        Vec2::new(80.0, 80.0),
        Vec2::new(260.0, 200.0),
        "Panel".to_string(),
    );
    panel.add_child(
        Button::new(
            Vec2::ZERO,
            Vec2::new(120.0, 36.0),
            "Inner Action".to_string(),
        ),
        Vec2::new(24.0, 32.0),
    );

    let child_position = panel.child(0).expect("panel should have child").position();
    let click_point = Vec2::new(child_position.x + 6.0, child_position.y + 8.0);

    panel.handle_event(&UiEvent::CursorMoved {
        position: click_point,
    });

    panel.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: click_point,
    });

    let release_event = panel.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Released,
        position: click_point,
    });

    match release_event {
        Some(WidgetEvent::ButtonClicked { label }) => assert_eq!(label, "Inner Action"),
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn panel_drag_moves_children() {
    let mut panel = Panel::new(
        Vec2::new(120.0, 90.0),
        Vec2::new(240.0, 180.0),
        "Panel".to_string(),
    );
    panel.add_child(
        Label::new(
            Vec2::ZERO,
            Vec2::new(60.0, 18.0),
            "Inside".to_string(),
            colors::ACCENT_SOFT,
        ),
        Vec2::new(18.0, 24.0),
    );

    let initial_child_pos = panel.child(0).expect("panel should have child").position();

    let drag_start = Vec2::new(panel.position().x + 10.0, panel.position().y + 10.0);
    panel.start_drag(drag_start);
    panel.update_drag(drag_start + Vec2::new(45.0, 55.0));
    panel.stop_drag();

    let moved_child_pos = panel.child(0).expect("panel should have child").position();

    assert_eq!(moved_child_pos, initial_child_pos + Vec2::new(45.0, 55.0));
}

#[test]
fn horizontal_layout_positions_children() {
    let mut layout = HorizontalLayout::new(Vec2::new(10.0, 20.0))
        .with_padding(Vec2::new(4.0, 6.0))
        .with_spacing(5.0)
        .with_cross_alignment(CrossAlignment::Center);

    layout.add_child(Label::new(
        Vec2::ZERO,
        Vec2::new(40.0, 16.0),
        "Label".to_string(),
        colors::BLUE,
    ));
    layout.add_child(Button::new(
        Vec2::ZERO,
        Vec2::new(24.0, 30.0),
        "Btn".to_string(),
    ));

    let first = layout.child(0).expect("layout should have first child");
    let second = layout.child(1).expect("layout should have second child");

    assert_eq!(first.position(), Vec2::new(14.0, 33.0));
    assert_eq!(second.position(), Vec2::new(59.0, 26.0));
    assert_eq!(layout.size(), Vec2::new(77.0, 42.0));
}

#[test]
fn vertical_layout_forwards_button_click() {
    let mut layout = VerticalLayout::new(Vec2::ZERO).with_spacing(4.0);

    layout.add_child(Checkbox::new(
        Vec2::ZERO,
        Vec2::new(20.0, 20.0),
        "Check".to_string(),
    ));
    layout.add_child(Button::new(
        Vec2::ZERO,
        Vec2::new(80.0, 24.0),
        "Action".to_string(),
    ));

    let click_point = Vec2::new(18.0, 44.0);
    layout.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: click_point,
    });

    let release_event = layout.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Released,
        position: click_point,
    });

    match release_event {
        Some(WidgetEvent::ButtonClicked { label }) => assert_eq!(label, "Action"),
        other => panic!("Unexpected event: {:?}", other),
    }
}

#[test]
fn remote_commands_update_button() {
    let channel = RemoteCommandChannel::new();
    channel.push(RemoteCommand {
        id: "primary".to_string(),
        method: "set_position".to_string(),
        params: json!({ "x": 48.0, "y": 64.0 }),
    });
    channel.push(RemoteCommand {
        id: "primary".to_string(),
        method: "set_size".to_string(),
        params: json!({ "width": 140.0, "height": 36.0 }),
    });
    channel.push(RemoteCommand {
        id: "primary".to_string(),
        method: "set_label".to_string(),
        params: json!({ "text": "Remote".to_string() }),
    });

    let mut button = Button::new(Vec2::ZERO, Vec2::new(80.0, 24.0), "Initial".to_string());

    let report = RemoteUiSession::new(&channel)
        .with_button("primary", &mut button)
        .process();

    assert!(
        report.errors.is_empty(),
        "unexpected remote errors: {:?}",
        report.errors
    );
    assert_eq!(report.processed, 3);
    assert_eq!(button.position(), Vec2::new(48.0, 64.0));
    assert_eq!(button.size(), Vec2::new(140.0, 36.0));
    assert_eq!(button.label(), "Remote");
}

#[test]
fn remote_unknown_target_reports_error() {
    let channel = RemoteCommandChannel::new();
    channel.push(RemoteCommand {
        id: "missing".to_string(),
        method: "set_position".to_string(),
        params: json!({ "x": 10.0, "y": 10.0 }),
    });

    let mut checkbox = Checkbox::new(Vec2::ZERO, Vec2::new(18.0, 18.0), "Check".to_string());
    let report = RemoteUiSession::new(&channel)
        .with_checkbox("existing", &mut checkbox)
        .process();

    assert_eq!(report.processed, 0);
    assert_eq!(report.errors.len(), 1);
    match &report.errors[0] {
        RemoteError::UnknownTarget { id, method } => {
            assert_eq!(id, "missing");
            assert_eq!(method, "set_position");
        }
        other => panic!("unexpected error variant: {:?}", other),
    }
}

#[test]
fn remote_host_attaches_button_to_layout() {
    let channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(channel.clone());

    channel.push(RemoteCommand {
        id: "layout".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "vertical_layout",
            "position": { "x": 0.0, "y": 0.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "button".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "button",
            "label": "Remote Button",
            "size": { "width": 120.0, "height": 36.0 }
        }),
    });
    let report = host.process();
    assert!(report.errors.is_empty());
    assert_eq!(report.processed, 2);

    channel.push(RemoteCommand {
        id: "layout".to_string(),
        method: "attach_child".to_string(),
        params: json!({ "child": "button" }),
    });

    let report = host.process();
    assert!(report.errors.is_empty());
    assert_eq!(report.processed, 1);
    assert!(host.is_attached_to("button", "layout"));

    let (mut events, cursor_hit) = host.handle_event(&UiEvent::CursorMoved {
        position: Vec2::new(12.0, 12.0),
    });
    assert!(
        cursor_hit,
        "cursor move inside button bounds should register a hit"
    );

    let (pressed_events, pressed_hit) = host.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(12.0, 12.0),
    });
    assert!(
        pressed_hit,
        "mouse press inside button bounds should register a hit"
    );
    events.extend(pressed_events);

    let (released_events, released_hit) = host.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Released,
        position: Vec2::new(12.0, 12.0),
    });
    assert!(
        released_hit,
        "mouse release inside button bounds should register a hit"
    );
    events.extend(released_events);

    assert!(
        events.iter().any(|event| matches!(
            event,
            WidgetEvent::ButtonClicked { label } if label == "Remote Button"
        )),
        "attached button did not emit click event"
    );

    let (_, outside_hit) = host.handle_event(&UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: Vec2::new(999.0, 999.0),
    });
    assert!(
        !outside_hit,
        "mouse press outside all widgets should not register a hit"
    );
}

#[test]
fn remote_host_updates_label_in_nested_layout() {
    let channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(channel.clone());

    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "panel",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 240.0, "height": 180.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "layout".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "vertical_layout",
            "position": { "x": 0.0, "y": 0.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "attach_child".to_string(),
        params: json!({ "child": "layout" }),
    });
    channel.push(RemoteCommand {
        id: "status".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "label",
            "text": "Initial",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 160.0, "height": 24.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "layout".to_string(),
        method: "attach_child".to_string(),
        params: json!({ "child": "status" }),
    });
    channel.push(RemoteCommand {
        id: "status".to_string(),
        method: "set_text".to_string(),
        params: json!({ "text": "Updated" }),
    });

    let report = host.process();
    assert!(
        report.errors.is_empty(),
        "unexpected remote errors: {:?}",
        report.errors
    );
    assert_eq!(report.processed, 6);
}

#[test]
fn remote_updates_layout_parameters() {
    let channel = RemoteCommandChannel::new();
    channel.push(RemoteCommand {
        id: "layout".to_string(),
        method: "set_spacing".to_string(),
        params: json!({ "value": 22.0 }),
    });
    channel.push(RemoteCommand {
        id: "layout".to_string(),
        method: "set_padding".to_string(),
        params: json!({ "x": 12.0, "y": 16.0 }),
    });
    channel.push(RemoteCommand {
        id: "layout".to_string(),
        method: "set_cross_alignment".to_string(),
        params: json!({ "alignment": "center" }),
    });

    let mut layout = HorizontalLayout::new(Vec2::ZERO);
    layout.add_child(Label::new(
        Vec2::ZERO,
        Vec2::new(40.0, 16.0),
        "L1".to_string(),
        colors::BLUE,
    ));

    let report = RemoteUiSession::new(&channel)
        .with_horizontal_layout("layout", &mut layout)
        .process();

    assert!(report.errors.is_empty());
    assert_eq!(report.processed, 3);
    assert_eq!(layout.len(), 1);
    assert_eq!(layout.size().y > 0.0, true);
}

#[test]
fn remote_host_creates_and_updates_widgets() {
    let channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(channel.clone());

    channel.push(RemoteCommand {
        id: "title".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "label",
            "text": "Remote UI",
            "position": { "x": 12.0, "y": 18.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "title".to_string(),
        method: "set_position".to_string(),
        params: json!({ "x": 24.0, "y": 32.0 }),
    });

    let report = host.process();
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert_eq!(report.processed, 2);
    assert!(host.contains("title"));
    assert_eq!(host.len(), 1);

    channel.push(RemoteCommand {
        id: "title".to_string(),
        method: "destroy".to_string(),
        params: json!({}),
    });

    let report = host.process();
    assert!(report.errors.is_empty());
    assert_eq!(report.processed, 1);
    assert!(!host.contains("title"));
    assert_eq!(host.len(), 0);
}

#[test]
fn remote_host_rejects_duplicate_ids() {
    let channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(channel.clone());

    channel.push(RemoteCommand {
        id: "btn".to_string(),
        method: "create".to_string(),
        params: json!({ "kind": "button" }),
    });
    channel.push(RemoteCommand {
        id: "btn".to_string(),
        method: "create".to_string(),
        params: json!({ "kind": "button" }),
    });

    let report = host.process();
    assert_eq!(report.processed, 1);
    assert_eq!(report.errors.len(), 1);
    match &report.errors[0] {
        RemoteError::AlreadyExists { id, method } => {
            assert_eq!(id, "btn");
            assert_eq!(method, "create");
        }
        other => panic!("unexpected error variant: {:?}", other),
    }
    assert!(host.contains("btn"));
}

#[test]
fn remote_host_reports_focused_widget() {
    let channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(channel.clone());

    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "panel",
            "title": "Controls",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 200.0, "height": 120.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "input".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "textbox",
            "position": { "x": 12.0, "y": 18.0 },
            "size": { "width": 160.0, "height": 28.0 }
        }),
    });

    let report = host.process();
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert!(!host.has_focused_widget());

    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "attach_child".to_string(),
        params: json!({ "child": "input" }),
    });

    let report = host.process();
    assert!(
        report.errors.is_empty(),
        "unexpected errors when attaching: {:?}",
        report.errors
    );
    assert!(!host.has_focused_widget());

    channel.push(RemoteCommand {
        id: "input".to_string(),
        method: "set_focused".to_string(),
        params: json!({ "value": true }),
    });

    let report = host.process();
    assert!(report.errors.is_empty());
    assert!(host.has_focused_widget());

    channel.push(RemoteCommand {
        id: "input".to_string(),
        method: "set_focused".to_string(),
        params: json!({ "value": false }),
    });

    let report = host.process();
    assert!(report.errors.is_empty());
    assert!(!host.has_focused_widget());
}

#[test]
fn remote_host_clear_all_removes_widgets() {
    let channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(channel.clone());

    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "panel",
            "title": "Control",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 200.0, "height": 150.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "status".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "label",
            "text": "Ready"
        }),
    });

    let report = host.process();
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert_eq!(report.processed, 2);
    assert_eq!(host.len(), 2);

    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "attach_child".to_string(),
        params: json!({
            "child": "status"
        }),
    });

    let report = host.process();
    assert!(
        report.errors.is_empty(),
        "unexpected errors when attaching: {:?}",
        report.errors
    );
    assert_eq!(report.processed, 1);
    assert!(host.contains("status"));

    channel.push(RemoteCommand {
        id: "_host".to_string(),
        method: "clear_all".to_string(),
        params: json!({}),
    });

    let report = host.process();
    assert!(report.errors.is_empty());
    assert_eq!(report.processed, 1);
    assert_eq!(host.len(), 0);
    assert!(!host.contains("status"));
    assert!(!host.contains("panel"));

    channel.push(RemoteCommand {
        id: "status".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "label",
            "text": "Cleared"
        }),
    });

    let report = host.process();
    assert!(report.errors.is_empty());
    assert_eq!(report.processed, 1);
    assert!(host.contains("status"));
    assert_eq!(host.len(), 1);
}

#[test]
fn remote_host_mouse_hit_includes_panel_children() {
    let channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(channel.clone());

    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "panel",
            "title": "Settings",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 180.0, "height": 80.0 }
        }),
    });
    channel.push(RemoteCommand {
        id: "dropdown".to_string(),
        method: "create".to_string(),
        params: json!({
            "kind": "dropdown",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 120.0, "height": 34.0 },
            "options": ["One", "Two", "Three", "Four", "Five", "Six"]
        }),
    });

    let report = host.process();
    assert!(report.errors.is_empty());

    channel.push(RemoteCommand {
        id: "panel".to_string(),
        method: "attach_child".to_string(),
        params: json!({
            "child": "dropdown"
        }),
    });

    let report = host.process();
    assert!(report.errors.is_empty());

    channel.push(RemoteCommand {
        id: "dropdown".to_string(),
        method: "set_open".to_string(),
        params: json!({ "value": true }),
    });

    let report = host.process();
    assert!(report.errors.is_empty());

    let dropdown_list_position = Vec2::new(24.0, 120.0);
    let event = UiEvent::MouseButton {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        position: dropdown_list_position,
    };

    let (_, mouse_hit) = host.handle_event(&event);
    assert!(
        mouse_hit,
        "expected a click on a dropdown list extending beyond the panel to count as a hit"
    );
}
