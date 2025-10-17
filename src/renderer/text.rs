use crate::primitives::{Shader, Texture, VertexArray, VertexBuffer};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use glam::{Mat4, Vec2, Vec4};
use std::mem;

pub struct TextRenderer {
    font: Font,
    px: f32,
    shader: Shader,
    vao: VertexArray,
    _vbo: VertexBuffer,
}

impl TextRenderer {
    pub fn from_bytes(font_bytes: &[u8], px: f32) -> Result<Self, String> {
        let font = Font::from_bytes(font_bytes, fontdue::FontSettings::default())
            .map_err(|e| format!("Failed to load font: {:?}", e))?;

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

            uniform sampler2D atlas;
            uniform vec4 color;

            void main() {
                float alpha = texture(atlas, TexCoord).a;
                FragColor = vec4(color.rgb, color.a * alpha);
            }
        "#;

        let shader = Shader::new(vertex_src, fragment_src)?;

        // Shared quad geometry: two triangles (TRIANGLE_FAN) with texcoords
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
        vao.set_attribute(0, 2, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as i32, 0);
        vao.set_attribute(1, 2, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as i32, 2 * mem::size_of::<f32>());
        vao.unbind();

        Ok(TextRenderer { font, px, shader, vao, _vbo: vbo })
    }

    pub fn set_projection(&self, projection: &Mat4) {
        self.shader.use_program();
        self.shader.set_mat4("projection", projection);
        self.shader.set_int("atlas", 0);
    }

    pub fn measure(&self, text: &str) -> Vec2 {
        if text.is_empty() { return Vec2::ZERO; }
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings::default());
        layout.append(&[&self.font], &TextStyle::new(text, self.px, 0));
        if layout.glyphs().is_empty() { return Vec2::ZERO; }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for g in layout.glyphs() {
            min_x = min_x.min(g.x);
            min_y = min_y.min(g.y);
            max_x = max_x.max(g.x + g.width as f32);
            max_y = max_y.max(g.y + g.height as f32);
        }
        Vec2::new((max_x - min_x).ceil().max(0.0), (max_y - min_y).ceil().max(0.0))
    }

    fn rasterize_rgba(&self, text: &str) -> (Vec<u8>, u32, u32) {
        if text.is_empty() { return (vec![], 1, 1); }
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings::default());
        layout.append(&[&self.font], &TextStyle::new(text, self.px, 0));

        if layout.glyphs().is_empty() { return (vec![], 1, 1); }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for g in layout.glyphs() {
            min_x = min_x.min(g.x);
            min_y = min_y.min(g.y);
            max_x = max_x.max(g.x + g.width as f32);
            max_y = max_y.max(g.y + g.height as f32);
        }
        let width = (max_x - min_x).ceil().max(1.0) as u32;
        let height = (max_y - min_y).ceil().max(1.0) as u32;
        let offset_x = -min_x;
        let offset_y = -min_y;

        let mut buffer = vec![0u8; (width * height * 4) as usize];
        for g in layout.glyphs() {
            let (metrics, bitmap) = self.font.rasterize_indexed(g.key.glyph_index, self.px);
            let gw = metrics.width as i32;
            let gh = metrics.height as i32;
            let dst_x0 = (g.x + offset_x).floor() as i32;
            let dst_y0 = (g.y + offset_y).floor() as i32;
            for y in 0..gh {
                for x in 0..gw {
                    let src_alpha = bitmap[(y * gw + x) as usize];
                    let dx = dst_x0 + x;
                    let dy = dst_y0 + y;
                    if dx >= 0 && dy >= 0 && (dx as u32) < width && (dy as u32) < height {
                        let idx = ((dy as u32 * width + dx as u32) * 4) as usize;
                        buffer[idx + 0] = 255;
                        buffer[idx + 1] = 255;
                        buffer[idx + 2] = 255;
                        buffer[idx + 3] = src_alpha;
                    }
                }
            }
        }
        (buffer, width, height)
    }

    pub fn draw_text(&self, position: Vec2, color: Vec4, text: &str) {
        if text.is_empty() { return; }
        let (rgba, w, h) = self.rasterize_rgba(text);
        let tex = Texture::from_data(w, h, &rgba);

        self.shader.use_program();
        self.shader.set_vec2("position", &position);
        self.shader.set_vec2("size", &Vec2::new(w as f32, h as f32));
        self.shader.set_vec4("color", &color);
        self.shader.set_int("atlas", 0);

        self.vao.bind();
        tex.bind(0);
        unsafe { gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4); }
        tex.unbind();
        self.vao.unbind();
        // Texture drops automatically
    }
}
