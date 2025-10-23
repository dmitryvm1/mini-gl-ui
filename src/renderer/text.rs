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
    // Cached line metrics (baseline ascent and descent in pixels)
    line_ascent: f32,
    line_descent: f32,
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
            0.0, 1.0, 0.0, 1.0, // top left
            1.0, 1.0, 1.0, 1.0, // top right
            1.0, 0.0, 1.0, 0.0, // bottom right
            0.0, 0.0, 0.0, 0.0, // bottom left
        ];

        let vao = VertexArray::new();
        let vbo = VertexBuffer::new();

        vao.bind();
        vbo.set_data(&vertices, gl::STATIC_DRAW);
        vao.set_attribute(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            4 * mem::size_of::<f32>() as i32,
            0,
        );
        vao.set_attribute(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            4 * mem::size_of::<f32>() as i32,
            2 * mem::size_of::<f32>(),
        );
        vao.unbind();

        // Cache line metrics so widgets can align text consistently regardless of glyph mix.
        let (line_ascent, line_descent) = if let Some(metrics) = font.horizontal_line_metrics(px) {
            let ascent = metrics.ascent.max(0.0);
            let descent = (-metrics.descent).max(0.0);
            (ascent, descent.max(0.0))
        } else {
            // Fallback to sampling a string with tall ascenders and deep descenders.
            let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
            layout.reset(&LayoutSettings::default());
            let sample = "HITMWAgjpyq";
            layout.append(&[&font], &TextStyle::new(sample, px, 0));
            if let Some(lines) = layout.lines() {
                if let Some(line) = lines.first() {
                    let ascent = line.max_ascent.max(0.0);
                    let descent = (-line.min_descent).max(0.0);
                    (ascent, descent)
                } else {
                    (px, px * 0.2)
                }
            } else {
                (px, px * 0.2)
            }
        };

        Ok(TextRenderer {
            font,
            px,
            shader,
            vao,
            _vbo: vbo,
            line_ascent,
            line_descent,
        })
    }

    pub fn set_projection(&self, projection: &Mat4) {
        self.shader.use_program();
        self.shader.set_mat4("projection", projection);
        self.shader.set_int("atlas", 0);
    }

    pub fn measure(&self, text: &str) -> Vec2 {
        if text.is_empty() {
            return Vec2::ZERO;
        }
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings::default());
        layout.append(&[&self.font], &TextStyle::new(text, self.px, 0));
        let glyphs = layout.glyphs();
        if glyphs.is_empty() {
            return Vec2::ZERO;
        }
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y_actual = f32::MAX;
        let mut max_y_actual = f32::MIN;
        for g in glyphs {
            min_x = min_x.min(g.x);
            max_x = max_x.max(g.x + g.width as f32);
            min_y_actual = min_y_actual.min(g.y);
            max_y_actual = max_y_actual.max(g.y + g.height as f32);
        }

        let mut top = min_y_actual;
        let mut bottom = max_y_actual;
        if let Some(lines) = layout.lines() {
            if let Some(line) = lines.first() {
                let baseline = line.baseline_y;
                top = top.min(baseline - self.line_ascent);
                bottom = bottom.max(baseline + self.line_descent);
            }
        }
        Vec2::new(
            (max_x - min_x).ceil().max(0.0),
            (bottom - top).ceil().max(0.0),
        )
    }

    /// Returns the distance in pixels from the top of the rasterized bounds to the text baseline
    pub fn baseline_offset(&self, text: &str) -> f32 {
        if text.is_empty() {
            return self.line_ascent;
        }
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings::default());
        layout.append(&[&self.font], &TextStyle::new(text, self.px, 0));
        let glyphs = layout.glyphs();
        if glyphs.is_empty() {
            return self.line_ascent;
        }
        let mut top = glyphs
            .iter()
            .fold(f32::MAX, |acc, g| if g.y < acc { g.y } else { acc });
        if let Some(lines) = layout.lines() {
            if let Some(line) = lines.first() {
                let baseline = line.baseline_y;
                top = top.min(baseline - self.line_ascent);
                let offset = (baseline - top).max(0.0);
                if offset > 0.0 {
                    return offset;
                }
            }
        }
        self.line_ascent
    }

    /// Returns cached line metrics (ascent above baseline, descent below baseline)
    pub fn line_metrics(&self) -> (f32, f32) {
        (self.line_ascent, self.line_descent)
    }

    fn rasterize_rgba(&self, text: &str) -> (Vec<u8>, u32, u32) {
        if text.is_empty() {
            return (vec![], 1, 1);
        }
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings::default());
        layout.append(&[&self.font], &TextStyle::new(text, self.px, 0));

        if layout.glyphs().is_empty() {
            return (vec![], 1, 1);
        }
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y_actual = f32::MAX;
        let mut max_y_actual = f32::MIN;
        for g in layout.glyphs() {
            min_x = min_x.min(g.x);
            max_x = max_x.max(g.x + g.width as f32);
            min_y_actual = min_y_actual.min(g.y);
            max_y_actual = max_y_actual.max(g.y + g.height as f32);
        }

        let mut top = min_y_actual;
        let mut bottom = max_y_actual;
        if let Some(lines) = layout.lines() {
            if let Some(line) = lines.first() {
                let baseline = line.baseline_y;
                // Anchor the raster bounds around the cached line metrics so the baseline stays fixed,
                // while still covering any glyph pixels that extend beyond those metrics.
                top = (baseline - self.line_ascent).min(top);
                bottom = (baseline + self.line_descent).max(bottom);
            }
        }
        let offset_y = -top;
        let width = (max_x - min_x).ceil().max(1.0) as u32;
        let height = (bottom - top).ceil().max(1.0) as u32;
        let offset_x = -min_x;

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
        if text.is_empty() {
            return;
        }
        let (rgba, w, h) = self.rasterize_rgba(text);
        let tex = Texture::from_data(w, h, &rgba);

        self.shader.use_program();
        self.shader.set_vec2("position", &position);
        self.shader.set_vec2("size", &Vec2::new(w as f32, h as f32));
        self.shader.set_vec4("color", &color);
        self.shader.set_int("atlas", 0);

        self.vao.bind();
        tex.bind(0);
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }
        tex.unbind();
        self.vao.unbind();
        // Texture drops automatically
    }
}
