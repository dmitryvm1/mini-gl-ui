use mini_gl_ui::{colors, renderer::QuadRenderer, ui::*, Vec2, Vec4};
use glam::Mat4;
use glfw::{Action, Context, Key, MouseButton};

fn main() {
    // Initialize GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    
    // Configure GLFW
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    
    // Create window
    let (mut window, events) = glfw
        .create_window(800, 600, "Mini GL UI Demo", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.make_current();
    
    // Load OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    
    // Enable blending for transparency
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    
    // Create renderer
    let renderer = QuadRenderer::new().expect("Failed to create quad renderer");
    
    // Set up orthographic projection
    let projection = Mat4::orthographic_rh_gl(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
    renderer.set_projection(&projection);
    
    // Create UI components
    let mut label = Label::new(
        Vec2::new(50.0, 50.0),
        Vec2::new(200.0, 40.0),
        "Label".to_string(),
        colors::BLUE,
    );
    
    let mut button = Button::new(
        Vec2::new(50.0, 110.0),
        Vec2::new(150.0, 40.0),
        "Click Me".to_string(),
    );
    
    let mut checkbox = Checkbox::new(
        Vec2::new(50.0, 170.0),
        Vec2::new(30.0, 30.0),
        "Checkbox".to_string(),
    );
    
    let mut textbox = TextBox::new(
        Vec2::new(50.0, 220.0),
        Vec2::new(200.0, 40.0),
        "Enter text...".to_string(),
    );
    
    let mut panel = Panel::new(
        Vec2::new(300.0, 50.0),
        Vec2::new(400.0, 300.0),
        "Draggable Panel".to_string(),
    ).with_colors(colors::WHITE, Vec4::new(0.2, 0.4, 0.6, 1.0));
    
    let mut mouse_pos = Vec2::ZERO;
    
    // Main loop
    while !window.should_close() {
        // Process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    mouse_pos = Vec2::new(x as f32, y as f32);
                    
                    // Update button hover state
                    button.set_hovered(button.contains_point(mouse_pos));
                    
                    // Update panel drag
                    panel.update_drag(mouse_pos);
                }
                glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                    // Check button click
                    if button.contains_point(mouse_pos) {
                        button.set_pressed(true);
                        println!("Button clicked!");
                    }
                    
                    // Check checkbox click
                    if checkbox.contains_point(mouse_pos) {
                        checkbox.toggle();
                        println!("Checkbox is now: {}", checkbox.is_checked());
                    }
                    
                    // Check textbox click
                    if textbox.contains_point(mouse_pos) {
                        textbox.set_focused(true);
                        println!("TextBox focused");
                    } else {
                        textbox.set_focused(false);
                    }
                    
                    // Check panel title bar for dragging
                    if panel.title_bar_contains_point(mouse_pos) {
                        panel.start_drag(mouse_pos);
                        println!("Started dragging panel");
                    }
                }
                glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                    button.set_pressed(false);
                    panel.stop_drag();
                }
                glfw::WindowEvent::Key(key, _, Action::Press, _) => {
                    if textbox.is_focused() {
                        match key {
                            Key::Backspace => textbox.backspace(),
                            Key::A => textbox.insert_char('a'),
                            Key::B => textbox.insert_char('b'),
                            Key::C => textbox.insert_char('c'),
                            Key::D => textbox.insert_char('d'),
                            Key::E => textbox.insert_char('e'),
                            Key::Space => textbox.insert_char(' '),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Clear the screen
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        
        // Draw UI components
        label.draw(&renderer);
        button.draw(&renderer);
        checkbox.draw(&renderer);
        textbox.draw(&renderer);
        panel.draw(&renderer);
        
        // Swap buffers
        window.swap_buffers();
    }
}
