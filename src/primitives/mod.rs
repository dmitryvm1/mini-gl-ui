//! OpenGL primitives that wrap low-level OpenGL API into simpler constructs

mod buffer;
mod shader;
mod texture;

pub use buffer::{VertexArray, VertexBuffer};
pub use shader::Shader;
pub use texture::Texture;
