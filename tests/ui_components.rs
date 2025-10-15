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
