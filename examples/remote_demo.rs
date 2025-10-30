use glam::Mat4;
use glfw::{Action, Context, Key, MouseButton, WindowEvent};
use mini_gl_ui::{
    colors,
    renderer::QuadRenderer,
    ui::{
        ButtonState as UiButtonState, KeyCode as UiKeyCode, MouseButton as UiMouseButton,
        RemoteCommand, RemoteCommandChannel, RemoteUiHost, UiEvent, WidgetEvent,
    },
    Vec2,
};
use serde_json::json;
use std::{
    fs,
    io::{self, BufRead},
    sync::mpsc,
    thread,
};

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 640;
fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("failed to initialize GLFW");
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            "mini-gl-ui Remote Demo",
            glfw::WindowMode::Windowed,
        )
        .expect("unable to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_char_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);
    window.set_close_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let mut renderer = QuadRenderer::new().expect("failed to create quad renderer");
    let projection = Mat4::orthographic_rh_gl(
        0.0,
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
        0.0,
        -1.0,
        1.0,
    );
    renderer.set_projection(&projection);
    configure_font(&mut renderer);

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let command_channel = RemoteCommandChannel::new();
    let mut host = RemoteUiHost::new(command_channel.clone());

    let (stdin_sender, stdin_receiver) = mpsc::channel::<String>();
    let _stdin_listener = command_channel.spawn_json_channel_listener(stdin_receiver);

    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if stdin_sender.send(line).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("stdin reader failed: {err}");
                    break;
                }
            }
        }
    });

    println!("Remote demo running.");
    println!("Type or pipe newline-delimited JSON commands into this terminal.");
    println!(
        "Example: {sample}",
        sample = r#"{"id":"status","method":"set_text","params":{"text":"hello from channel"}}"#
    );
    println!(
        "Attach example: {}",
        r#"{"id":"control_layout","method":"attach_child","params":{"child":"example_child"}}"#
    );

    bootstrap_scene(&command_channel);
    let init_report = host.process();
    if !init_report.errors.is_empty() {
        for error in init_report.errors {
            eprintln!("Remote init error: {error}");
        }
    }

    let mut mouse_pos = Vec2::ZERO;
    let mut spawned_labels = 0usize;
    let mut debug_overlay_enabled = false;

    while !window.should_close() {
        glfw.poll_events();

        let mut emitted_events = Vec::new();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Close => window.set_should_close(true),
                WindowEvent::CursorPos(x, y) => {
                    mouse_pos = Vec2::new(x as f32, y as f32);
                    let (events, _) = host.handle_event(&UiEvent::CursorMoved {
                        position: mouse_pos,
                    });
                    emitted_events.extend(events);
                }
                WindowEvent::MouseButton(button, action, _) => {
                    if let Some(ui_button) = translate_button(button) {
                        let state = match action {
                            Action::Press => UiButtonState::Pressed,
                            Action::Release => UiButtonState::Released,
                            _ => continue,
                        };
                        let (events, _) = host.handle_event(&UiEvent::MouseButton {
                            button: ui_button,
                            state,
                            position: mouse_pos,
                        });
                        emitted_events.extend(events);
                    }
                }
                WindowEvent::Scroll(_, y) => {
                    let (events, _) = host.handle_event(&UiEvent::Scroll {
                        delta: y as f32,
                        position: mouse_pos,
                    });
                    emitted_events.extend(events);
                }
                WindowEvent::Char(character) => {
                    let (events, _) = host.handle_event(&UiEvent::CharacterInput(character));
                    emitted_events.extend(events);
                }
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                WindowEvent::Key(Key::Backspace, _, action, _)
                    if matches!(action, Action::Press | Action::Repeat) =>
                {
                    let (events, _) = host.handle_event(&UiEvent::KeyInput {
                        key: UiKeyCode::Backspace,
                    });
                    emitted_events.extend(events);
                }
                _ => {}
            }
        }

        for widget_event in emitted_events {
            match widget_event {
                WidgetEvent::ButtonClicked { id: _, label } => {
                    if label == "Spawn Remote Label" {
                        spawned_labels += 1;
                        spawn_dynamic_label(&command_channel, spawned_labels);
                        update_status(
                            &command_channel,
                            format!("Spawned dynamic label #{spawned_labels}"),
                        );
                    } else {
                        update_status(&command_channel, format!("Button '{label}' clicked"));
                    }
                }
                WidgetEvent::CheckboxToggled {
                    id: _,
                    label,
                    checked,
                } => {
                    if label == "Enable debug overlay" {
                        debug_overlay_enabled = checked;
                    }
                    update_status(
                        &command_channel,
                        format!(
                            "{label} toggled: {}",
                            if checked { "enabled" } else { "disabled" }
                        ),
                    );
                }
                WidgetEvent::TextChanged { id: _, text } => {
                    update_status(&command_channel, format!("Typed: {text}"));
                }
                WidgetEvent::TextBoxFocusChanged { id: _, focused } => {
                    let message = if focused {
                        "Text box focused".to_string()
                    } else {
                        "Text box unfocused".to_string()
                    };
                    update_status(&command_channel, message);
                }
                WidgetEvent::DropdownSelectionChanged { id, selected } => {
                    if id == "remote_action_dropdown" {
                        handle_remote_action_dropdown(
                            &command_channel,
                            selected.as_str(),
                            &mut spawned_labels,
                            &mut debug_overlay_enabled,
                        );
                    } else {
                        update_status(&command_channel, format!("Dropdown {id} -> {selected}"));
                    }
                }
                _ => {}
            }
        }

        let report = host.process();
        if !report.errors.is_empty() {
            for error in report.errors {
                eprintln!("Remote error: {error}");
            }
        }

        unsafe {
            gl::ClearColor(0.07, 0.08, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        renderer.draw_rect(
            Vec2::ZERO,
            Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            colors::SURFACE_DARK,
        );
        host.draw(&renderer);
        window.swap_buffers();
    }
}

fn configure_font(renderer: &mut QuadRenderer) {
    let candidates = [
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/System/Library/Fonts/SFNSDisplay.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ];

    for path in candidates {
        if let Ok(bytes) = fs::read(path) {
            if renderer.set_font_from_bytes(&bytes, 14.0).is_ok() {
                println!("Loaded font from {path}");
                return;
            }
        }
    }

    println!("No compatible system font found; text rendering will be skipped.");
}

fn translate_button(button: MouseButton) -> Option<UiMouseButton> {
    match button {
        MouseButton::Button1 => Some(UiMouseButton::Left),
        MouseButton::Button2 => Some(UiMouseButton::Right),
        MouseButton::Button3 => Some(UiMouseButton::Middle),
        _ => None,
    }
}

fn enqueue(channel: &RemoteCommandChannel, id: &str, method: &str, params: serde_json::Value) {
    channel.push(RemoteCommand {
        id: id.to_string(),
        method: method.to_string(),
        params,
    });
}

fn update_status(channel: &RemoteCommandChannel, message: String) {
    enqueue(channel, "status", "set_text", json!({ "text": message }));
}

fn spawn_dynamic_label(channel: &RemoteCommandChannel, index: usize) {
    let new_id = format!("spawned_label_{index}");
    let label_text = format!("Dynamic label #{index}");
    enqueue(
        channel,
        &new_id,
        "create",
        json!({
            "kind": "label",
            "text": label_text,
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 260.0, "height": 26.0 },
            "color": { "r": 0.35, "g": 0.58, "b": 0.92, "a": 0.88 }
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "attach_child",
        json!({ "child": new_id }),
    );
}

fn handle_remote_action_dropdown(
    channel: &RemoteCommandChannel,
    action: &str,
    spawned_labels: &mut usize,
    debug_overlay_enabled: &mut bool,
) {
    match action {
        "Inspect state" => {
            let overlay_state = if *debug_overlay_enabled {
                "enabled"
            } else {
                "disabled"
            };
            update_status(
                channel,
                format!(
                    "Remote state: {} dynamic labels, debug overlay {}",
                    *spawned_labels, overlay_state
                ),
            );
        }
        "Spawn label burst" => {
            let burst = 3;
            for _ in 0..burst {
                *spawned_labels += 1;
                spawn_dynamic_label(channel, *spawned_labels);
            }
            update_status(
                channel,
                format!(
                    "Spawned {} remote labels (total {})",
                    burst, *spawned_labels
                ),
            );
        }
        "Toggle debug overlay" => {
            let new_state = !*debug_overlay_enabled;
            enqueue(
                channel,
                "debug_toggle",
                "set_checked",
                json!({ "value": new_state }),
            );
            *debug_overlay_enabled = new_state;
            update_status(
                channel,
                format!(
                    "Debug overlay {} via dropdown",
                    if new_state { "enabled" } else { "disabled" }
                ),
            );
        }
        "Focus text box" => {
            enqueue(
                channel,
                "message_box",
                "set_focused",
                json!({ "value": true }),
            );
            update_status(channel, "Message box focused via dropdown".to_string());
        }
        "Reset status message" => {
            update_status(channel, "Ready. Waiting for stdin commands".to_string());
        }
        other => {
            update_status(
                channel,
                format!("Dropdown remote_action_dropdown -> {}", other),
            );
        }
    }
}

fn bootstrap_scene(channel: &RemoteCommandChannel) {
    enqueue(
        channel,
        "control_panel",
        "create",
        json!({
            "kind": "panel",
            "title": "Remote Console",
            "position": { "x": 36.0, "y": 32.0 },
            "size": { "width": 360.0, "height": 320.0 }
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "create",
        json!({
            "kind": "vertical_layout",
            "position": { "x": 0.0, "y": 0.0 }
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "set_padding",
        json!({ "x": 12.0, "y": 14.0 }),
    );
    enqueue(
        channel,
        "control_layout",
        "set_spacing",
        json!({ "value": 12.0 }),
    );
    enqueue(
        channel,
        "control_panel",
        "attach_child",
        json!({
            "child": "control_layout",
            "offset": { "x": 24.0, "y": 72.0 }
        }),
    );
    enqueue(
        channel,
        "status",
        "create",
        json!({
            "kind": "label",
            "text": "Waiting for commands...",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 280.0, "height": 28.0 },
            "color": { "r": 0.46, "g": 0.72, "b": 0.96, "a": 0.85 }
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "attach_child",
        json!({ "child": "status" }),
    );
    enqueue(
        channel,
        "spawn_button",
        "create",
        json!({
            "kind": "button",
            "label": "Spawn Remote Label",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 220.0, "height": 36.0 }
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "attach_child",
        json!({ "child": "spawn_button" }),
    );
    enqueue(
        channel,
        "remote_action_dropdown",
        "create",
        json!({
            "kind": "dropdown",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 240.0, "height": 34.0 },
            "placeholder": "Select a remote action",
            "options": [
                "Inspect state",
                "Spawn label burst",
                "Toggle debug overlay",
                "Focus text box",
                "Reset status message"
            ],
            "max_visible_items": 5
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "attach_child",
        json!({ "child": "remote_action_dropdown" }),
    );
    enqueue(
        channel,
        "debug_toggle",
        "create",
        json!({
            "kind": "checkbox",
            "label": "Enable debug overlay",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 180.0, "height": 28.0 }
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "attach_child",
        json!({ "child": "debug_toggle" }),
    );
    enqueue(
        channel,
        "message_box",
        "create",
        json!({
            "kind": "textbox",
            "position": { "x": 0.0, "y": 0.0 },
            "size": { "width": 240.0, "height": 32.0 },
            "placeholder": "Type to update the status label"
        }),
    );
    enqueue(
        channel,
        "control_layout",
        "attach_child",
        json!({ "child": "message_box" }),
    );
    enqueue(
        channel,
        "status",
        "set_text",
        json!({
            "text": "Ready. Waiting for stdin commands"
        }),
    );
}
