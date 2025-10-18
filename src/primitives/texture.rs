use gl::types::*;
use std::ffi::c_void;
use std::ptr;

/// A wrapper around OpenGL textures
pub struct Texture {
    pub id: GLuint,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /// Creates a new empty texture
    pub fn new(width: u32, height: u32) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            // Allocate texture storage
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Texture { id, width, height }
    }

    /// Creates a texture from RGBA data
    pub fn from_data(width: u32, height: u32, data: &[u8]) -> Self {
        let texture = Self::new(width, height);
        texture.update_data(data);
        texture
    }

    /// Updates the texture data
    pub fn update_data(&self, data: &[u8]) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    /// Binds this texture to a texture unit
    pub fn bind(&self, unit: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    /// Unbinds the texture
    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
