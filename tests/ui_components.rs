use mini_gl_ui::{colors, ui::*, Vec2};

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
