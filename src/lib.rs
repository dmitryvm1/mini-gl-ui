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
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};
    use std::sync::RwLock;

    /// Named palette slots that can be customized at runtime.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum PaletteSlot {
        TextPrimary,
        TextSecondary,
        SurfaceDark,
        Surface,
        SurfaceLight,
        Accent,
        AccentSoft,
        BorderSoft,
        BorderSubtle,
        Checkmark,
        Shadow,
    }

    /// Palette containing the primary UI styling colors.
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Palette {
        pub text_primary: Vec4,
        pub text_secondary: Vec4,
        pub surface_dark: Vec4,
        pub surface: Vec4,
        pub surface_light: Vec4,
        pub accent: Vec4,
        pub accent_soft: Vec4,
        pub border_soft: Vec4,
        pub border_subtle: Vec4,
        pub checkmark: Vec4,
        pub shadow: Vec4,
    }

    impl Palette {
        /// Creates a new palette with the provided colors.
        pub const fn new(
            text_primary: Vec4,
            text_secondary: Vec4,
            surface_dark: Vec4,
            surface: Vec4,
            surface_light: Vec4,
            accent: Vec4,
            accent_soft: Vec4,
            border_soft: Vec4,
            border_subtle: Vec4,
            checkmark: Vec4,
            shadow: Vec4,
        ) -> Self {
            Self {
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
            }
        }

        /// Returns the color stored at the provided palette slot.
        pub fn get(&self, slot: PaletteSlot) -> Vec4 {
            match slot {
                PaletteSlot::TextPrimary => self.text_primary,
                PaletteSlot::TextSecondary => self.text_secondary,
                PaletteSlot::SurfaceDark => self.surface_dark,
                PaletteSlot::Surface => self.surface,
                PaletteSlot::SurfaceLight => self.surface_light,
                PaletteSlot::Accent => self.accent,
                PaletteSlot::AccentSoft => self.accent_soft,
                PaletteSlot::BorderSoft => self.border_soft,
                PaletteSlot::BorderSubtle => self.border_subtle,
                PaletteSlot::Checkmark => self.checkmark,
                PaletteSlot::Shadow => self.shadow,
            }
        }

        /// Updates the color stored at the provided palette slot.
        pub fn set(&mut self, slot: PaletteSlot, value: Vec4) {
            match slot {
                PaletteSlot::TextPrimary => self.text_primary = value,
                PaletteSlot::TextSecondary => self.text_secondary = value,
                PaletteSlot::SurfaceDark => self.surface_dark = value,
                PaletteSlot::Surface => self.surface = value,
                PaletteSlot::SurfaceLight => self.surface_light = value,
                PaletteSlot::Accent => self.accent = value,
                PaletteSlot::AccentSoft => self.accent_soft = value,
                PaletteSlot::BorderSoft => self.border_soft = value,
                PaletteSlot::BorderSubtle => self.border_subtle = value,
                PaletteSlot::Checkmark => self.checkmark = value,
                PaletteSlot::Shadow => self.shadow = value,
            }
        }
    }

    impl Default for Palette {
        fn default() -> Self {
            Self::new(
                Vec4::new(0.95, 0.98, 1.0, 1.0),
                Vec4::new(0.8, 0.85, 0.92, 1.0),
                Vec4::new(0.08, 0.1, 0.14, 0.7),
                Vec4::new(0.16, 0.22, 0.32, 0.78),
                Vec4::new(0.24, 0.32, 0.44, 0.85),
                Vec4::new(0.35, 0.58, 0.92, 0.92),
                Vec4::new(0.46, 0.72, 0.96, 0.65),
                Vec4::new(0.58, 0.72, 0.9, 0.95),
                Vec4::new(0.12, 0.16, 0.22, 0.85),
                Vec4::new(0.4, 0.78, 0.52, 0.95),
                Vec4::new(0.0, 0.0, 0.0, 0.35),
            )
        }
    }

    static GLOBAL_PALETTE: Lazy<RwLock<Palette>> = Lazy::new(|| RwLock::new(Palette::default()));

    fn palette_cell() -> &'static RwLock<Palette> {
        &GLOBAL_PALETTE
    }

    /// Returns the active palette.
    pub fn palette() -> Palette {
        *palette_cell()
            .read()
            .expect("palette lock poisoned while reading")
    }

    /// Replaces the active palette with the provided one.
    pub fn set_palette(palette: Palette) {
        let mut guard = palette_cell()
            .write()
            .expect("palette lock poisoned while writing");
        *guard = palette;
    }

    /// Applies a closure to mutate the active palette in place.
    pub fn update_palette<F>(mutator: F)
    where
        F: FnOnce(&mut Palette),
    {
        if let Ok(mut guard) = palette_cell().write() {
            mutator(&mut guard);
        }
    }

    /// Sets a single palette slot to a new value, returning the previous color.
    pub fn set_palette_slot(slot: PaletteSlot, value: Vec4) -> Vec4 {
        let mut guard = palette_cell()
            .write()
            .expect("palette lock poisoned while updating slot");
        let previous = guard.get(slot);
        guard.set(slot, value);
        previous
    }

    /// Returns the color stored in the provided palette slot.
    pub fn palette_color(slot: PaletteSlot) -> Vec4 {
        palette().get(slot)
    }

    pub const WHITE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Vec4 = Vec4::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Vec4 = Vec4::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Vec4 = Vec4::new(0.0, 0.0, 1.0, 1.0);
    pub const GRAY: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GRAY: Vec4 = Vec4::new(0.8, 0.8, 0.8, 1.0);
    pub const DARK_GRAY: Vec4 = Vec4::new(0.3, 0.3, 0.3, 1.0);

    /// Primary text color.
    pub fn text_primary() -> Vec4 {
        palette().text_primary
    }

    /// Secondary text color used for muted labels.
    pub fn text_secondary() -> Vec4 {
        palette().text_secondary
    }

    pub fn surface_dark() -> Vec4 {
        palette().surface_dark
    }

    pub fn surface() -> Vec4 {
        palette().surface
    }

    pub fn surface_light() -> Vec4 {
        palette().surface_light
    }

    pub fn accent() -> Vec4 {
        palette().accent
    }

    pub fn accent_soft() -> Vec4 {
        palette().accent_soft
    }

    pub fn border_soft() -> Vec4 {
        palette().border_soft
    }

    pub fn border_subtle() -> Vec4 {
        palette().border_subtle
    }

    pub fn checkmark() -> Vec4 {
        palette().checkmark
    }

    pub fn shadow() -> Vec4 {
        palette().shadow
    }
}
