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
}
