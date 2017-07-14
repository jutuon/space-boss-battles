/*
gl/src/gl_wrapper/uniform.rs, 2017-07-14

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use super::gl_raw;
use self::gl_raw::types::*;

use cgmath::{Vector3, Matrix4};
use cgmath::prelude::*;

use std::ffi::CString;

use gl_wrapper::shader::Program;

#[derive(Debug)]
pub enum UniformError {
    UniformNotFoundOrGLPrefix,
}

pub trait CreateUniform
    where Self: Sized {

    fn new(name: CString, program: &Program) -> Result<Self, UniformError> {
        let location;

        unsafe {
            location = gl_raw::GetUniformLocation(program.id(), name.as_ptr());
        }

        if location == -1 {
            Err(UniformError::UniformNotFoundOrGLPrefix)
        } else {
            unsafe {
                Ok(Self::from_location(location))
            }
        }

    }

    unsafe fn from_location(location: GLint) -> Self;
}


pub struct UniformVector3 {
    location: GLint,
}

impl CreateUniform for UniformVector3 {
    unsafe fn from_location(location: GLint) -> UniformVector3 {
        UniformVector3 {location}
    }
}

impl UniformVector3 {
    pub fn send(&self, data: &Vector3<f32>) {
        unsafe {
            gl_raw::Uniform3fv(self.location, 1, data.as_ptr());
        }
    }
}

pub struct UniformMatrix4 {
    location: GLint,
}

impl CreateUniform for UniformMatrix4 {
    unsafe fn from_location(location: GLint) -> UniformMatrix4 {
        UniformMatrix4 {location}
    }
}

impl UniformMatrix4 {
    pub fn send(&self, data: &Matrix4<f32>) {
        unsafe {
            gl_raw::UniformMatrix4fv(self.location, 1, gl_raw::FALSE, data.as_ptr());
        }
    }
}
