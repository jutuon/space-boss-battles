/*
gl/src/gl_wrapper/uniform.rs, 2017-07-19

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Send specific values to shader `Program`.

use super::gl_raw;
use self::gl_raw::types::*;

use cgmath::{Vector3, Matrix4};
use cgmath::prelude::*;

use std::ffi::CString;

use gl_wrapper::shader::Program;

/// Error information about uniform.
#[derive(Debug)]
pub enum UniformError {
    UniformNotFoundOrGLPrefix,
}

/// Common functionality between different types of uniforms.
pub trait Uniform
    where Self: Sized {

    type Data;

    /// Create new uniform.
    ///
    /// # Arguments
    /// * `name` - Name of the uniform.
    /// * `program` - Uniform's shader program.
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

    /// Create uniform from index, which is returned
    /// from OpenGL's function `GetUniformLocation`.
    /// This function does not check if the index is valid, so
    /// this function is marked as unsafe.
    unsafe fn from_location(location: GLint) -> Self;

    /// Sends data to shader. You have to make sure that the
    /// `Program` object which contains the uniform is currently
    /// enabled with it's `use_program` method.
    fn send(&mut self, data: &Self::Data);
}

/// Uniform for Vector3
pub struct UniformVector3 {
    location: GLint,
}

impl Uniform for UniformVector3 {
    type Data = Vector3<f32>;

    unsafe fn from_location(location: GLint) -> UniformVector3 {
        UniformVector3 {location}
    }

    fn send(&mut self, data: &Self::Data) {
        unsafe {
            gl_raw::Uniform3fv(self.location, 1, data.as_ptr());
        }
    }
}

/// Uniform for Matrix4
pub struct UniformMatrix4 {
    location: GLint,
}

impl Uniform for UniformMatrix4 {
    type Data = Matrix4<f32>;

    unsafe fn from_location(location: GLint) -> UniformMatrix4 {
        UniformMatrix4 {location}
    }

    fn send(&mut self, data: &Self::Data) {
        unsafe {
            gl_raw::UniformMatrix4fv(self.location, 1, gl_raw::FALSE, data.as_ptr());
        }
    }
}