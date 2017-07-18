/*
gl/src/gl_wrapper/shader.rs, 2017-07-14

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Compile shaders and link program from them.

use super::gl_raw;
use self::gl_raw::types::*;

use std::ffi::{CString, IntoStringError};
use std::ptr;
use std::error::Error;


/// Type of shader
#[derive(Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    /// Return shader as `GLenum`. This is useful when calling functions
    /// from raw OpenGL bindings.
    fn as_gl_enum(self) -> GLenum {
        match self {
            ShaderType::Fragment => gl_raw::FRAGMENT_SHADER,
            ShaderType::Vertex => gl_raw::VERTEX_SHADER,
        }
    }
}

/// Compiled shader
pub struct Shader {
    shader_id: GLuint,
}

impl Shader {
    /// Compile shader. Returns compiled shader or error message.
    pub fn new(shader_type: ShaderType, shader_text: CString) -> Result<Shader, String> {
        let shader_type: GLenum = shader_type.as_gl_enum();
        let shader;

        unsafe {
            shader = Shader { shader_id: gl_raw::CreateShader(shader_type) };

            gl_raw::ShaderSource(shader.id(), 1, &shader_text.as_ptr(), ptr::null());
            gl_raw::CompileShader(shader.id());
        }

        let mut status: GLint = 0;

        unsafe {
            gl_raw::GetShaderiv(shader.id(), gl_raw::COMPILE_STATUS, &mut status);
        }

        if status == 0 {
            match Shader::get_shader_log(&shader) {
                Ok(message) => Err(message),
                Err(into_string_error) => Err(into_string_error.description().to_string()),
            }
        } else {
            Ok(shader)
        }
    }

    /// OpenGL's identification number for shader object.
    fn id(&self) -> GLuint {
        self.shader_id
    }

    /// Return compilation error log. Returns IntoStringError if error log from
    /// OpenGL is not valid string.
    fn get_shader_log(shader: &Shader) -> Result<String, IntoStringError> {
        let mut log_length: GLint = 0;

        unsafe {
            gl_raw::GetShaderiv(shader.id(), gl_raw::INFO_LOG_LENGTH, &mut log_length);
        }

        let buffer = create_string_buffer(log_length as usize).into_raw();

        unsafe {
            gl_raw::GetShaderInfoLog(shader.id(), log_length, ptr::null_mut(), buffer);
            CString::from_raw(buffer).into_string()
        }
    }
}

impl Drop for Shader {
    /// Deletes OpenGL's shader object.
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteShader(self.shader_id);
        }
    }
}

pub struct Program {
    program_id: GLuint,
}

impl Drop for Program {
    // Deletes OpenGL's program object.
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteProgram(self.program_id);
        }
    }
}

impl Program {
    /// Link new program from compiled shaders. Returns linked program or error message.
    pub fn new(shader1: Shader, shader2: Shader) -> Result<Program, String> {
        let program;

        unsafe {
            program = Program { program_id: gl_raw::CreateProgram() };

            gl_raw::AttachShader(program.id(), shader1.id());
            gl_raw::AttachShader(program.id(), shader2.id());
            gl_raw::LinkProgram(program.id());
        }

        let mut status: GLint = 0;

        unsafe {
            gl_raw::GetProgramiv(program.id(), gl_raw::LINK_STATUS, &mut status);
        }

        if status == 0 {
            match Program::get_program_log(&program) {
                Ok(message) => Err(message),
                Err(into_string_error) => Err(into_string_error.description().to_string()),
            }
        } else {
            Ok(program)
        }
    }

    /// Enable program for next rendering function call.
    pub fn use_program(&self){
        unsafe {
            gl_raw::UseProgram(self.program_id);
        }
    }

    /// OpenGL's identification number for program object.
    pub(crate) fn id(&self) -> GLuint {
        self.program_id
    }

    /// Return linking error log. Returns IntoStringError if error log from
    /// OpenGL is not valid string.
    fn get_program_log(program: &Program) -> Result<String, IntoStringError> {
        let mut log_length: GLint = 0;

        unsafe {
            gl_raw::GetProgramiv(program.id(), gl_raw::INFO_LOG_LENGTH, &mut log_length);
        }

        let buffer = create_string_buffer(log_length as usize).into_raw();

        unsafe {
            gl_raw::GetProgramInfoLog(program.id(), log_length, ptr::null_mut(), buffer );
            CString::from_raw(buffer).into_string()
        }
    }
}

/// Creates specific size CString.
///
/// # Panics
/// If function's internally created Vec<u8> buffer's
/// length and argument len not match this function panics.
/// This should never happen.
fn create_string_buffer(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len);

    for _ in 0..len {
        buffer.push(b' ');
    }

    if buffer.len() != len {
        panic!("buffer and log length differs");
    }

    CString::new(buffer).unwrap()
}

