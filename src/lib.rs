//! A minimal OpenGL UI engine for drawing user interfaces
//!
//! This library provides simple and easy-to-use primitives for building
//! OpenGL-based user interfaces with components like buttons, checkboxes,
//! text boxes, labels, and draggable panels.

pub mod primitives;
pub mod renderer;
pub mod ui;

pub use glam::{Vec2, Vec3, Vec4};

/// Common color definitions
pub mod colors {
    use super::Vec4;

    pub const WHITE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Vec4 = Vec4::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Vec4 = Vec4::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Vec4 = Vec4::new(0.0, 0.0, 1.0, 1.0);
    pub const GRAY: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GRAY: Vec4 = Vec4::new(0.8, 0.8, 0.8, 1.0);
    pub const DARK_GRAY: Vec4 = Vec4::new(0.3, 0.3, 0.3, 1.0);

    // UI styling palette
    pub const TEXT_PRIMARY: Vec4 = Vec4::new(0.95, 0.98, 1.0, 1.0);
    pub const TEXT_SECONDARY: Vec4 = Vec4::new(0.8, 0.85, 0.92, 1.0);

    pub const SURFACE_DARK: Vec4 = Vec4::new(0.08, 0.1, 0.14, 0.7);
    pub const SURFACE: Vec4 = Vec4::new(0.16, 0.22, 0.32, 0.78);
    pub const SURFACE_LIGHT: Vec4 = Vec4::new(0.24, 0.32, 0.44, 0.85);

    pub const ACCENT: Vec4 = Vec4::new(0.35, 0.58, 0.92, 0.92);
    pub const ACCENT_SOFT: Vec4 = Vec4::new(0.46, 0.72, 0.96, 0.65);
    pub const BORDER_SOFT: Vec4 = Vec4::new(0.58, 0.72, 0.9, 0.95);
    pub const BORDER_SUBTLE: Vec4 = Vec4::new(0.12, 0.16, 0.22, 0.85);
    pub const CHECKMARK: Vec4 = Vec4::new(0.4, 0.78, 0.52, 0.95);
    pub const SHADOW: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.35);
}
