//! OpenGL primitives that wrap low-level OpenGL API into simpler constructs

mod shader;
mod buffer;
mod texture;

pub use shader::Shader;
pub use buffer::{VertexBuffer, VertexArray};
pub use texture::Texture;
