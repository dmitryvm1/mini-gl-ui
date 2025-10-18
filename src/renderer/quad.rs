use crate::primitives::{Shader, Texture, VertexArray, VertexBuffer};
use crate::renderer::TextRenderer;
use glam::{Mat4, Vec2, Vec4};
use std::mem;

/// A renderer for drawing 2D quads (rectangles)
pub struct QuadRenderer {
    shader: Shader,
    vao: VertexArray,
    _vbo: VertexBuffer,
    text: Option<TextRenderer>,
    last_projection: Option<Mat4>,
}

impl QuadRenderer {
    /// Creates a new quad renderer
    pub fn new() -> Result<Self, String> {
        let vertex_src = r#"
            #version 330 core
            layout (location = 0) in vec2 aPos;
            layout (location = 1) in vec2 aTexCoord;
            
            uniform mat4 projection;
            uniform vec2 position;
            uniform vec2 size;
            
            out vec2 TexCoord;
            
            void main() {
                vec2 pos = aPos * size + position;
                gl_Position = projection * vec4(pos, 0.0, 1.0);
                TexCoord = aTexCoord;
            }
        "#;

        let fragment_src = r#"
            #version 330 core
            in vec2 TexCoord;
            out vec4 FragColor;
            
            uniform vec4 color;
            uniform sampler2D tex;
            uniform int use_texture;
            
            void main() {
                if (use_texture == 1) {
                    FragColor = texture(tex, TexCoord) * color;
                } else {
                    FragColor = color;
                }
            }
        "#;

        let shader = Shader::new(vertex_src, fragment_src)?;

        // Define quad vertices (position and texture coordinates)
        let vertices: [f32; 16] = [
            // positions   // tex coords
            0.0, 1.0, 0.0, 1.0, // top left
            1.0, 1.0, 1.0, 1.0, // top right
            1.0, 0.0, 1.0, 0.0, // bottom right
            0.0, 0.0, 0.0, 0.0, // bottom left
        ];

        let vao = VertexArray::new();
        let vbo = VertexBuffer::new();

        vao.bind();
        vbo.set_data(&vertices, gl::STATIC_DRAW);

        // Position attribute
        vao.set_attribute(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            4 * mem::size_of::<f32>() as i32,
            0,
        );
        // Texture coordinate attribute
        vao.set_attribute(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            4 * mem::size_of::<f32>() as i32,
            2 * mem::size_of::<f32>(),
        );

        vao.unbind();

        let renderer = QuadRenderer {
            shader,
            vao,
            _vbo: vbo,
            text: None,
            last_projection: None,
        };

        renderer.shader.use_program();
        renderer.shader.set_int("tex", 0);
        renderer.shader.set_int("use_texture", 0);

        Ok(renderer)
    }

    /// Sets the projection matrix
    pub fn set_projection(&mut self, projection: &Mat4) {
        self.shader.use_program();
        self.shader.set_mat4("projection", projection);
        if let Some(text) = &self.text {
            text.set_projection(projection);
        }
        self.last_projection = Some(*projection);
    }

    /// Draws a filled rectangle
    pub fn draw_rect(&self, position: Vec2, size: Vec2, color: Vec4) {
        self.shader.use_program();
        self.shader.set_vec2("position", &position);
        self.shader.set_vec2("size", &size);
        self.shader.set_int("use_texture", 0);
        self.shader.set_vec4("color", &color);

        self.vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }
        self.vao.unbind();
    }

    /// Draws a rectangle outline
    pub fn draw_rect_outline(&self, position: Vec2, size: Vec2, color: Vec4, thickness: f32) {
        // Top
        self.draw_rect(position, Vec2::new(size.x, thickness), color);
        // Bottom
        self.draw_rect(
            Vec2::new(position.x, position.y + size.y - thickness),
            Vec2::new(size.x, thickness),
            color,
        );
        // Left
        self.draw_rect(position, Vec2::new(thickness, size.y), color);
        // Right
        self.draw_rect(
            Vec2::new(position.x + size.x - thickness, position.y),
            Vec2::new(thickness, size.y),
            color,
        );
    }

    /// Draws a textured rectangle with an optional tint
    pub fn draw_textured_rect(&self, position: Vec2, size: Vec2, texture: &Texture, tint: Vec4) {
        self.shader.use_program();
        self.shader.set_vec2("position", &position);
        self.shader.set_vec2("size", &size);
        self.shader.set_vec4("color", &tint);
        self.shader.set_int("use_texture", 1);

        texture.bind(0);
        self.vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }
        self.vao.unbind();
        texture.unbind();

        self.shader.set_int("use_texture", 0);
    }

    /// Initializes a font for text rendering from bytes
    pub fn set_font_from_bytes(&mut self, font_bytes: &[u8], px: f32) -> Result<(), String> {
        let tr = TextRenderer::from_bytes(font_bytes, px)?;
        // If we already have a projection, apply it to the new text renderer
        if let Some(p) = self.last_projection {
            tr.set_projection(&p);
        }
        self.text = Some(tr);
        Ok(())
    }

    /// Draws text at the given position using the configured font
    pub fn draw_text(&self, position: Vec2, color: Vec4, text: &str) {
        if let Some(tr) = &self.text {
            tr.draw_text(position, color, text);
        }
    }

    /// Measures text dimensions using the configured font; returns zero when unavailable
    pub fn measure_text(&self, text: &str) -> Vec2 {
        if let Some(tr) = &self.text {
            tr.measure(text)
        } else {
            Vec2::ZERO
        }
    }

    /// Returns distance from top of text bounds to baseline for given text
    pub fn baseline_offset(&self, text: &str) -> f32 {
        if let Some(tr) = &self.text {
            tr.baseline_offset(text)
        } else {
            0.0
        }
    }

    /// Returns cached line metrics (ascent, descent) if a font is configured
    pub fn line_metrics(&self) -> Option<(f32, f32)> {
        if let Some(tr) = &self.text {
            Some(tr.line_metrics())
        } else {
            None
        }
    }
}
