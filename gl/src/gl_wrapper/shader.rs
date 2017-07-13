/*
gl/src/gl_wrapper/shader.rs, 2017-07-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use super::gl_raw;
use self::gl_raw::types::*;

use std::ffi::{CString, IntoStringError};
use std::ptr;
use std::error::Error;



#[derive(Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn as_gl_enum(self) -> GLenum {
        match self {
            ShaderType::Fragment => gl_raw::VERTEX_SHADER,
            ShaderType::Vertex => gl_raw::FRAGMENT_SHADER,
        }
    }
}

pub struct Shader {
    shader_id: GLuint,
}

impl Shader {
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

    fn id(&self) -> GLuint {
        self.shader_id
    }

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
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteProgram(self.program_id);
        }
    }
}

impl Program {
    pub fn new(shader1: Shader, shader2: Shader) -> Result<Program, String> {
        let program;

        unsafe {
            program = Program { program_id: gl_raw::CreateProgram() };

            gl_raw::AttachShader(shader1.id(), program.id());
            gl_raw::AttachShader(shader2.id(), program.id());
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

    pub fn use_program(&self){
        unsafe {
            gl_raw::UseProgram(self.program_id);
        }
    }

    pub(crate) fn id(&self) -> GLuint {
        self.program_id
    }

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

