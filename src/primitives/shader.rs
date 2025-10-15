use gl::types::*;
use std::ffi::CString;
use std::ptr;

/// A wrapper around OpenGL shader programs
pub struct Shader {
    pub id: GLuint,
}

impl Shader {
    /// Creates a new shader from vertex and fragment shader source code
    pub fn new(vertex_src: &str, fragment_src: &str) -> Result<Self, String> {
        unsafe {
            // Compile vertex shader
            let vertex_shader = Self::compile_shader(vertex_src, gl::VERTEX_SHADER)?;
            
            // Compile fragment shader
            let fragment_shader = Self::compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;
            
            // Link shaders into a program
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
            
            // Check for linking errors
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);
                buffer.set_len(len as usize - 1);
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                return Err(String::from_utf8_unchecked(buffer));
            }
            
            // Clean up shaders (they're linked into the program now)
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            
            Ok(Shader { id: program })
        }
    }
    
    /// Compiles a shader from source code
    unsafe fn compile_shader(src: &str, shader_type: GLenum) -> Result<GLuint, String> {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        
        // Check for compilation errors
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = Vec::with_capacity(len as usize);
            buffer.set_len(len as usize - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buffer.as_mut_ptr() as *mut GLchar,
            );
            return Err(String::from_utf8_unchecked(buffer));
        }
        
        Ok(shader)
    }
    
    /// Activates this shader program
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
    
    /// Sets a uniform mat4 value
    pub fn set_mat4(&self, name: &str, value: &glam::Mat4) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ref().as_ptr());
        }
    }
    
    /// Sets a uniform vec4 value
    pub fn set_vec4(&self, name: &str, value: &glam::Vec4) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform4f(location, value.x, value.y, value.z, value.w);
        }
    }
    
    /// Sets a uniform vec2 value
    pub fn set_vec2(&self, name: &str, value: &glam::Vec2) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform2f(location, value.x, value.y);
        }
    }
    
    /// Sets a uniform float value
    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform1f(location, value);
        }
    }
    
    /// Sets a uniform int value
    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform1i(location, value);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
