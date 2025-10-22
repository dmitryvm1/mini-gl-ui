use glam::Mat4;
use glfw::{Action, Context, Key, MouseButton};
use mini_gl_ui::{
    colors,
    primitives::Texture,
    renderer::QuadRenderer,
    ui::{
        Button, ButtonState as UiButtonState, Checkbox, CrossAlignment, Dropdown, HorizontalLayout,
        KeyCode as UiKeyCode, Label, MouseButton as UiMouseButton, Panel, TextBox, UiEvent,
        VerticalLayout, Widget, WidgetEvent,
    },
    Vec2, Vec4,
};
use std::f32::consts::PI;

fn blend_pixel(pixels: &mut [u8], width: u32, height: u32, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 || x as u32 >= width || y as u32 >= height {
        return;
    }

    let idx = ((y as u32 * width + x as u32) * 4) as usize;
    let dst = &mut pixels[idx..idx + 4];
    let alpha = color[3] as f32 / 255.0;
    if alpha <= 0.0 {
        return;
    }
    for i in 0..3 {
        let orig = dst[i] as f32;
        let blended = orig * (1.0 - alpha) + color[i] as f32 * alpha;
        dst[i] = blended.clamp(0.0, 255.0).round() as u8;
    }
    let new_alpha = dst[3] as f32 + color[3] as f32 * (1.0 - dst[3] as f32 / 255.0);
    dst[3] = new_alpha.clamp(0.0, 255.0).round() as u8;
}

fn draw_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    origin_x: i32,
    origin_y: i32,
    rect_width: u32,
    rect_height: u32,
    color: [u8; 4],
) {
    for dy in 0..rect_height {
        for dx in 0..rect_width {
            blend_pixel(
                pixels,
                width,
                height,
                origin_x + dx as i32,
                origin_y + dy as i32,
                color,
            );
        }
    }
}

fn draw_rect_outline(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    origin_x: i32,
    origin_y: i32,
    rect_width: u32,
    rect_height: u32,
    thickness: u32,
    color: [u8; 4],
) {
    draw_rect(
        pixels, width, height, origin_x, origin_y, rect_width, thickness, color,
    );
    draw_rect(
        pixels,
        width,
        height,
        origin_x,
        origin_y + rect_height as i32 - thickness as i32,
        rect_width,
        thickness,
        color,
    );
    draw_rect(
        pixels,
        width,
        height,
        origin_x,
        origin_y,
        thickness,
        rect_height,
        color,
    );
    draw_rect(
        pixels,
        width,
        height,
        origin_x + rect_width as i32 - thickness as i32,
        origin_y,
        thickness,
        rect_height,
        color,
    );
}

fn draw_circle(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: [u8; 4],
) {
    let radius_sq = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius_sq {
                blend_pixel(pixels, width, height, center_x + dx, center_y + dy, color);
            }
        }
    }
}

fn create_mmo_background_texture(width: u32, height: u32) -> Texture {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let horizon = (height as f32 * 0.62) as u32;

    let sky_top = [26.0, 38.0, 92.0];
    let sky_bottom = [196.0, 143.0, 96.0];
    let ground_top = [86.0, 64.0, 42.0];
    let ground_bottom = [56.0, 44.0, 36.0];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            if y < horizon {
                let t = y as f32 / horizon as f32;
                let r = sky_top[0] * (1.0 - t) + sky_bottom[0] * t;
                let g = sky_top[1] * (1.0 - t) + sky_bottom[1] * t;
                let b = sky_top[2] * (1.0 - t) + sky_bottom[2] * t;
                pixels[idx] = r.clamp(0.0, 255.0) as u8;
                pixels[idx + 1] = g.clamp(0.0, 255.0) as u8;
                pixels[idx + 2] = b.clamp(0.0, 255.0) as u8;
            } else {
                let t = (y - horizon) as f32 / (height - horizon).max(1) as f32;
                let r = ground_top[0] * (1.0 - t) + ground_bottom[0] * t;
                let g = ground_top[1] * (1.0 - t) + ground_bottom[1] * t;
                let b = ground_top[2] * (1.0 - t) + ground_bottom[2] * t;
                pixels[idx] = r.clamp(0.0, 255.0) as u8;
                pixels[idx + 1] = g.clamp(0.0, 255.0) as u8;
                pixels[idx + 2] = b.clamp(0.0, 255.0) as u8;
            }
            pixels[idx + 3] = 255;
        }
    }

    // Distant mountain silhouette
    let far_base = (horizon as i32 - (height as f32 * 0.08) as i32).max(0);
    for x in 0..width {
        let normalized = x as f32 / width as f32;
        let peak = (height as f32 * (0.32 - 0.08 * (normalized * PI * 2.5).sin())).round() as i32;
        let peak = peak.max(0);
        let base = far_base.max(peak + 1);
        for y in peak..base {
            let blend_t = (y - peak) as f32 / (base - peak).max(1) as f32;
            let shade = (120.0 * (1.0 - blend_t) + 70.0 * blend_t).round() as u8;
            blend_pixel(
                &mut pixels,
                width,
                height,
                x as i32,
                y,
                [shade.saturating_sub(20), shade, shade + 20, 255],
            );
        }
    }

    // Foreground peaks
    let near_base = (horizon as i32 + (height as f32 * 0.02) as i32).min(height as i32 - 1);
    for x in 0..width {
        let normalized = x as f32 / width as f32;
        let peak = (height as f32 * (0.45 - 0.12 * (normalized * PI * 3.3 + 0.7).cos())).round();
        let peak = peak as i32;
        let base = near_base.max(peak + 1);
        for y in peak..base {
            let blend_t = (y - peak) as f32 / (base - peak).max(1) as f32;
            let r = 90.0 * (1.0 - blend_t) + 60.0 * blend_t;
            let g = 100.0 * (1.0 - blend_t) + 72.0 * blend_t;
            let b = 82.0 * (1.0 - blend_t) + 60.0 * blend_t;
            blend_pixel(
                &mut pixels,
                width,
                height,
                x as i32,
                y,
                [r as u8, g as u8, b as u8, 255],
            );
        }
    }

    // Sun and glow
    let sun_center_x = (width as f32 * 0.75) as i32;
    let sun_center_y = (height as f32 * 0.18) as i32;
    draw_circle(
        &mut pixels,
        width,
        height,
        sun_center_x,
        sun_center_y,
        (height as f32 * 0.08) as i32,
        [255, 229, 180, 255],
    );
    draw_circle(
        &mut pixels,
        width,
        height,
        sun_center_x,
        sun_center_y,
        (height as f32 * 0.11) as i32,
        [255, 220, 160, 90],
    );
    draw_circle(
        &mut pixels,
        width,
        height,
        sun_center_x,
        sun_center_y,
        (height as f32 * 0.14) as i32,
        [255, 196, 128, 40],
    );

    // Stylized clouds
    let cloud_color = [235, 240, 255, 180];
    let cloud_positions = [
        (width as f32 * 0.18, height as f32 * 0.22),
        (width as f32 * 0.36, height as f32 * 0.15),
        (width as f32 * 0.55, height as f32 * 0.26),
    ];
    for (cx, cy) in cloud_positions {
        let cx = cx as i32;
        let cy = cy as i32;
        for offset in [(-35, 0), (10, -10), (30, 15), (-10, 20), (40, 5)] {
            draw_circle(
                &mut pixels,
                width,
                height,
                cx + offset.0,
                cy + offset.1,
                (height as f32 * 0.045) as i32,
                cloud_color,
            );
        }
    }

    // Ground path
    draw_rect(
        &mut pixels,
        width,
        height,
        (width as f32 * 0.28) as i32,
        horizon as i32,
        (width as f32 * 0.44) as u32,
        (height as f32 * 0.38) as u32,
        [140, 110, 70, 60],
    );

    // Characters
    let party_positions = [
        (width as f32 * 0.4, horizon as f32 + height as f32 * 0.04),
        (width as f32 * 0.47, horizon as f32 + height as f32 * 0.03),
        (width as f32 * 0.54, horizon as f32 + height as f32 * 0.05),
    ];
    let character_colors = [[220, 70, 70, 255], [70, 160, 220, 255], [220, 190, 70, 255]];
    for ((cx, cy), color) in party_positions.iter().zip(character_colors.iter()) {
        let base_x = *cx as i32;
        let base_y = *cy as i32;
        draw_rect(
            &mut pixels,
            width,
            height,
            base_x - 6,
            base_y - 32,
            12,
            28,
            *color,
        );
        draw_circle(
            &mut pixels,
            width,
            height,
            base_x,
            base_y - 38,
            8,
            [255, 224, 204, 255],
        );
        draw_rect(
            &mut pixels,
            width,
            height,
            base_x - 10,
            base_y - 10,
            20,
            12,
            [40, 32, 28, 200],
        );
    }

    // Enemy creature
    let enemy_x = (width as f32 * 0.68) as i32;
    let enemy_y = (horizon as f32 + height as f32 * 0.02) as i32;
    draw_rect(
        &mut pixels,
        width,
        height,
        enemy_x - 24,
        enemy_y - 40,
        48,
        36,
        [90, 30, 110, 255],
    );
    draw_circle(
        &mut pixels,
        width,
        height,
        enemy_x - 12,
        enemy_y - 44,
        8,
        [255, 96, 96, 255],
    );
    draw_circle(
        &mut pixels,
        width,
        height,
        enemy_x + 12,
        enemy_y - 44,
        8,
        [255, 96, 96, 255],
    );

    // UI: health and mana bars
    draw_rect(
        &mut pixels,
        width,
        height,
        24,
        24,
        (width as f32 * 0.22) as u32,
        48,
        [20, 26, 40, 180],
    );
    draw_rect(
        &mut pixels,
        width,
        height,
        34,
        34,
        (width as f32 * 0.2) as u32,
        14,
        [180, 50, 60, 220],
    );
    draw_rect(
        &mut pixels,
        width,
        height,
        34,
        52,
        (width as f32 * 0.13) as u32,
        10,
        [70, 130, 220, 220],
    );
    draw_rect_outline(
        &mut pixels,
        width,
        height,
        24,
        24,
        (width as f32 * 0.22) as u32,
        48,
        2,
        [255, 255, 255, 160],
    );

    // UI: minimap
    let minimap_center_x = width as i32 - 80;
    let minimap_center_y = 90;
    draw_rect(
        &mut pixels,
        width,
        height,
        minimap_center_x - 58,
        minimap_center_y - 58,
        116,
        116,
        [16, 24, 36, 220],
    );
    draw_circle(
        &mut pixels,
        width,
        height,
        minimap_center_x,
        minimap_center_y,
        52,
        [64, 122, 96, 255],
    );
    draw_circle(
        &mut pixels,
        width,
        height,
        minimap_center_x + 12,
        minimap_center_y - 18,
        10,
        [220, 200, 90, 255],
    );
    draw_rect(
        &mut pixels,
        width,
        height,
        minimap_center_x - 6,
        minimap_center_y - 6,
        12,
        12,
        [240, 80, 80, 255],
    );

    // UI: quest tracker
    draw_rect(
        &mut pixels,
        width,
        height,
        width as i32 - 220,
        200,
        180,
        120,
        [18, 22, 32, 200],
    );
    for i in 0..4 {
        draw_rect(
            &mut pixels,
            width,
            height,
            width as i32 - 206,
            212 + i * 26,
            12,
            12,
            [230, 210, 110, 255],
        );
        draw_rect(
            &mut pixels,
            width,
            height,
            width as i32 - 186,
            212 + i * 26,
            140,
            12,
            [235, 235, 240, 180],
        );
    }

    // UI: chat window
    draw_rect(
        &mut pixels,
        width,
        height,
        24,
        height as i32 - 140,
        260,
        120,
        [12, 18, 26, 200],
    );
    draw_rect(
        &mut pixels,
        width,
        height,
        24,
        height as i32 - 140,
        260,
        22,
        [30, 40, 54, 220],
    );
    for i in 0..5 {
        draw_rect(
            &mut pixels,
            width,
            height,
            36,
            height as i32 - 110 + i * 18,
            220,
            12,
            [200, 210, 240 - (i * 5) as u8, 160],
        );
    }

    // UI: action bar
    let bar_width = (width as f32 * 0.52) as u32;
    let bar_x = ((width - bar_width) / 2) as i32;
    draw_rect(
        &mut pixels,
        width,
        height,
        bar_x,
        height as i32 - 80,
        bar_width,
        56,
        [16, 20, 28, 220],
    );
    let slot_size = 40;
    let gap = 6;
    let slots = 8;
    let total_width = slots * slot_size + (slots - 1) * gap;
    let start_x = bar_x + ((bar_width as i32 - total_width as i32) / 2);
    for i in 0..slots {
        let slot_x = start_x + i as i32 * (slot_size + gap);
        draw_rect(
            &mut pixels,
            width,
            height,
            slot_x,
            height as i32 - 70,
            slot_size as u32,
            slot_size as u32,
            [48, 52, 62, 255],
        );
        draw_rect_outline(
            &mut pixels,
            width,
            height,
            slot_x,
            height as i32 - 70,
            slot_size as u32,
            slot_size as u32,
            2,
            [200, 200, 210, 220],
        );
        draw_rect(
            &mut pixels,
            width,
            height,
            slot_x + 6,
            height as i32 - 64,
            28,
            28,
            [
                (120 + (i * 10) as u8).min(255),
                (90 + (i * 7) as u8).min(255),
                (60 + (i * 5) as u8).min(255),
                255,
            ],
        );
    }

    Texture::from_data(width, height, &pixels)
}

fn dispatch_ui_event<'a>(widgets: Vec<&'a mut dyn Widget>, event: &UiEvent) -> Vec<WidgetEvent> {
    let mut generated = Vec::new();
    for widget in widgets {
        if let Some(widget_event) = widget.handle_event(event) {
            generated.push(widget_event);
        }
    }
    generated
}

fn draw_widgets(widgets: Vec<&dyn Widget>, renderer: &QuadRenderer) {
    for widget in widgets {
        widget.draw(renderer);
    }
}

fn main() {
    const WINDOW_WIDTH: u32 = 800;
    const WINDOW_HEIGHT: u32 = 600;
    // Initialize GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // Configure GLFW
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    // Create window
    let (mut window, events) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            "Mini GL UI Demo",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.set_key_polling(true);
    window.set_char_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.make_current();

    // Load OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const std::ffi::c_void);

    // Enable blending for transparency
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // Create renderer
    let mut renderer = QuadRenderer::new().expect("Failed to create quad renderer");

    // Set up orthographic projection
    let projection = Mat4::orthographic_rh_gl(
        0.0,
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
        0.0,
        -1.0,
        1.0,
    );
    renderer.set_projection(&projection);

    // Configure font for text rendering (Windows system fonts)
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        renderer.set_font_from_bytes(&bytes, 18.0).ok();
    } else if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\arial.ttf") {
        renderer.set_font_from_bytes(&bytes, 18.0).ok();
    }

    // Create UI components
    let mut label = Label::new(
        Vec2::new(50.0, 50.0),
        Vec2::new(200.0, 40.0),
        "Label \n multiline".to_string(),
        colors::ACCENT_SOFT,
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

    let mut dropdown = Dropdown::new(
        Vec2::new(50.0, 280.0),
        Vec2::new(220.0, 36.0),
        "class_select".to_string(),
        vec![
            "Warrior".to_string(),
            "Mage".to_string(),
            "Rogue".to_string(),
            "Cleric".to_string(),
            "Ranger".to_string(),
            "Paladin".to_string(),
            "Bard".to_string(),
            "Druid".to_string(),
        ],
    )
    .with_placeholder("Choose a class".to_string())
    .with_max_visible_items(4);

    let mut panel = Panel::new(
        Vec2::new(300.0, 50.0),
        Vec2::new(400.0, 300.0),
        "Draggable Panel".to_string(),
    )
    .with_colors(colors::SURFACE_DARK, colors::ACCENT)
    .with_padding(Vec2::new(18.0, 16.0));

    let mut panel_controls_row = HorizontalLayout::new(Vec2::ZERO).with_spacing(12.0);
    panel_controls_row.add_child(Button::new(
        Vec2::ZERO,
        Vec2::new(120.0, 34.0),
        "Apply Buff".to_string(),
    ));
    panel_controls_row.add_child(Button::new(
        Vec2::ZERO,
        Vec2::new(120.0, 34.0),
        "Clear Buff".to_string(),
    ));

    let mut panel_layout = VerticalLayout::new(Vec2::ZERO)
        .with_spacing(12.0)
        .with_cross_alignment(CrossAlignment::Start);
    panel_layout.add_child(Label::new(
        Vec2::ZERO,
        Vec2::new(260.0, 28.0),
        "Raid Controls".to_string(),
        colors::ACCENT_SOFT,
    ));
    panel_layout.add_child(Label::new(
        Vec2::ZERO,
        Vec2::new(300.0, 24.0),
        "Toggle quick options directly inside the panel:".to_string(),
        colors::TEXT_SECONDARY,
    ));
    panel_layout.add_child(Checkbox::new(
        Vec2::ZERO,
        Vec2::new(26.0, 26.0),
        "Enable overlay markers".to_string(),
    ));
    panel_layout.add_child(Checkbox::new(
        Vec2::ZERO,
        Vec2::new(26.0, 26.0),
        "Lock panel position".to_string(),
    ));
    panel_layout.add_child(panel_controls_row);

    panel.add_child(panel_layout, Vec2::ZERO);

    let mut action_row = HorizontalLayout::new(Vec2::ZERO).with_spacing(12.0);
    action_row.add_child(Button::new(
        Vec2::ZERO,
        Vec2::new(120.0, 36.0),
        "Accept".to_string(),
    ));
    action_row.add_child(Button::new(
        Vec2::ZERO,
        Vec2::new(120.0, 36.0),
        "Decline".to_string(),
    ));

    let mut layout_section = VerticalLayout::new(Vec2::new(320.0, 380.0))
        .with_spacing(14.0)
        .with_cross_alignment(CrossAlignment::Center);
    layout_section.add_child(Label::new(
        Vec2::ZERO,
        Vec2::new(260.0, 32.0),
        "Party Actions".to_string(),
        colors::ACCENT_SOFT,
    ));
    layout_section.add_child(action_row);
    layout_section.add_child(Checkbox::new(
        Vec2::ZERO,
        Vec2::new(26.0, 26.0),
        "Remember choice".to_string(),
    ));

    let background_texture = create_mmo_background_texture(WINDOW_WIDTH, WINDOW_HEIGHT);

    let mut mouse_pos = Vec2::ZERO;
    let mut typed_text = String::new();
    let mut selected_class = dropdown.selected().unwrap_or("None").to_string();
    label.set_text(format!("Text: {} Class: {}", typed_text, selected_class));

    // Main loop
    while !window.should_close() {
        // Process events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            let mut emitted_events = Vec::new();
            match event {
                glfw::WindowEvent::CursorPos(x, y) => {
                    mouse_pos = Vec2::new(x as f32, y as f32);
                    let ui_event = UiEvent::CursorMoved {
                        position: mouse_pos,
                    };
                    emitted_events = dispatch_ui_event(
                        vec![
                            &mut label,
                            &mut button,
                            &mut checkbox,
                            &mut textbox,
                            &mut dropdown,
                            &mut panel,
                            &mut layout_section,
                        ],
                        &ui_event,
                    );
                }
                glfw::WindowEvent::MouseButton(MouseButton::Button1, action, _) => {
                    let state = match action {
                        Action::Press => Some(UiButtonState::Pressed),
                        Action::Release => Some(UiButtonState::Released),
                        _ => None,
                    };
                    if let Some(state) = state {
                        let ui_event = UiEvent::MouseButton {
                            button: UiMouseButton::Left,
                            state,
                            position: mouse_pos,
                        };
                        emitted_events = dispatch_ui_event(
                            vec![
                                &mut label,
                                &mut button,
                                &mut checkbox,
                                &mut textbox,
                                &mut dropdown,
                                &mut panel,
                                &mut layout_section,
                            ],
                            &ui_event,
                        );
                    }
                }
                glfw::WindowEvent::Scroll(_, y) => {
                    let ui_event = UiEvent::Scroll {
                        delta: y as f32,
                        position: mouse_pos,
                    };
                    emitted_events = dispatch_ui_event(
                        vec![
                            &mut label,
                            &mut button,
                            &mut checkbox,
                            &mut textbox,
                            &mut dropdown,
                            &mut panel,
                            &mut layout_section,
                        ],
                        &ui_event,
                    );
                }
                glfw::WindowEvent::Char(character) => {
                    let ui_event = UiEvent::CharacterInput(character);
                    emitted_events = dispatch_ui_event(
                        vec![
                            &mut label,
                            &mut button,
                            &mut checkbox,
                            &mut textbox,
                            &mut dropdown,
                            &mut panel,
                            &mut layout_section,
                        ],
                        &ui_event,
                    );
                }
                glfw::WindowEvent::Key(key, _, action, _) => {
                    if key == Key::Escape && action == Action::Press {
                        window.set_should_close(true);
                    }
                    if matches!(action, Action::Press | Action::Repeat) {
                        let keycode = match key {
                            Key::Backspace => Some(UiKeyCode::Backspace),
                            _ => None,
                        };
                        if let Some(keycode) = keycode {
                            let ui_event = UiEvent::KeyInput { key: keycode };
                            emitted_events = dispatch_ui_event(
                                vec![
                                    &mut label,
                                    &mut button,
                                    &mut checkbox,
                                    &mut textbox,
                                    &mut dropdown,
                                    &mut panel,
                                    &mut layout_section,
                                ],
                                &ui_event,
                            );
                        }
                    }
                }
                _ => {}
            }

            for widget_event in emitted_events {
                match widget_event {
                    WidgetEvent::ButtonClicked {
                        label: button_label,
                    } => {
                        println!("Button '{button_label}' clicked!");
                    }
                    WidgetEvent::CheckboxToggled {
                        label: checkbox_label,
                        checked,
                    } => {
                        println!("Checkbox '{checkbox_label}' is now: {checked}");
                    }
                    WidgetEvent::TextChanged { text } => {
                        typed_text = text;
                        label.set_text(format!("Text: {} | Class: {}", typed_text, selected_class));
                    }
                    WidgetEvent::TextBoxFocusChanged { focused } => {
                        println!("TextBox {}", if focused { "focused" } else { "unfocused" });
                    }
                    WidgetEvent::DropdownSelectionChanged { id, selected } => {
                        selected_class = selected;
                        label.set_text(format!("Text: {} | Class: {}", typed_text, selected_class));
                        println!("Dropdown '{id}' selected '{selected_class}'");
                    }
                    WidgetEvent::PanelDragStarted => println!("Started dragging panel"),
                    WidgetEvent::PanelDragged { .. } => {}
                    WidgetEvent::PanelDragEnded => println!("Stopped dragging panel"),
                }
            }
        }

        // Clear the screen
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Draw background scene
        renderer.draw_textured_rect(
            Vec2::ZERO,
            Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            &background_texture,
            Vec4::ONE,
        );

        // Draw UI components
        draw_widgets(
            vec![
                &label,
                &button,
                &checkbox,
                &textbox,
                &dropdown,
                &panel,
                &layout_section,
            ],
            &renderer,
        );

        // Swap buffers
        window.swap_buffers();
    }
}
