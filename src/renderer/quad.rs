use crate::primitives::{Shader, VertexArray, VertexBuffer};
use glam::{Mat4, Vec2, Vec4};
use std::mem;

/// A renderer for drawing 2D quads (rectangles)
pub struct QuadRenderer {
    shader: Shader,
    vao: VertexArray,
    _vbo: VertexBuffer,
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
            
            void main() {
                FragColor = color;
            }
        "#;
        
        let shader = Shader::new(vertex_src, fragment_src)?;
        
        // Define quad vertices (position and texture coordinates)
        let vertices: [f32; 16] = [
            // positions   // tex coords
            0.0, 1.0,      0.0, 1.0, // top left
            1.0, 1.0,      1.0, 1.0, // top right
            1.0, 0.0,      1.0, 0.0, // bottom right
            0.0, 0.0,      0.0, 0.0, // bottom left
        ];
        
        let vao = VertexArray::new();
        let vbo = VertexBuffer::new();
        
        vao.bind();
        vbo.set_data(&vertices, gl::STATIC_DRAW);
        
        // Position attribute
        vao.set_attribute(0, 2, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as i32, 0);
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
        
        Ok(QuadRenderer { shader, vao, _vbo: vbo })
    }
    
    /// Sets the projection matrix
    pub fn set_projection(&self, projection: &Mat4) {
        self.shader.use_program();
        self.shader.set_mat4("projection", projection);
    }
    
    /// Draws a filled rectangle
    pub fn draw_rect(&self, position: Vec2, size: Vec2, color: Vec4) {
        self.shader.use_program();
        self.shader.set_vec2("position", &position);
        self.shader.set_vec2("size", &size);
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
}
