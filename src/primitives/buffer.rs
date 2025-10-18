use gl::types::*;
use std::ffi::c_void;
use std::mem;

/// A wrapper around OpenGL Vertex Buffer Object (VBO)
pub struct VertexBuffer {
    pub id: GLuint,
}

impl VertexBuffer {
    /// Creates a new vertex buffer
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        VertexBuffer { id }
    }

    /// Binds this buffer
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    /// Uploads data to the buffer
    pub fn set_data<T>(&self, data: &[T], usage: GLenum) {
        self.bind();
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
                usage,
            );
        }
    }

    /// Unbinds the buffer
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

/// A wrapper around OpenGL Vertex Array Object (VAO)
pub struct VertexArray {
    pub id: GLuint,
}

impl VertexArray {
    /// Creates a new vertex array
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        VertexArray { id }
    }

    /// Binds this vertex array
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    /// Configures a vertex attribute
    pub fn set_attribute(
        &self,
        index: GLuint,
        size: GLint,
        data_type: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        offset: usize,
    ) {
        self.bind();
        unsafe {
            gl::EnableVertexAttribArray(index);
            gl::VertexAttribPointer(
                index,
                size,
                data_type,
                normalized,
                stride,
                offset as *const c_void,
            );
        }
    }

    /// Unbinds the vertex array
    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}
