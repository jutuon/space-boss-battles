/*
gl/src/gl_wrapper/mod.rs, 2017-08-24

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

pub mod gl_raw {

    //! Raw OpenGL bindings form `gl_generator` crate

    #[cfg(feature = "gles")]
    pub use gl_es_generated::*;

    #[cfg(not(feature = "gles"))]
    pub use gl_generated::*;
}

pub mod shader;
pub mod uniform;
pub mod buffer;
pub mod texture;


use gl_raw::types::*;

use std::os::raw::c_char;
use std::ffi::CStr;

/// OpenGL error types
#[derive(Debug)]
pub enum GLError {
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    InvalidFramebufferOperation,
    OutOfMemory,
    UnknownError(GLenum),
}

impl GLError {
    /// Get next error from OpenGL. Returns `Err(GLError)` if there is an error.
    pub fn get_error() -> Result<(),GLError> {
        let error;

        unsafe {
            error = gl_raw::GetError();
        }

        if error == gl_raw::NO_ERROR {
            return Ok(());
        }

        let error = match error {
            gl_raw::INVALID_ENUM => GLError::InvalidEnum,
            gl_raw::INVALID_VALUE => GLError::InvalidValue,
            gl_raw::INVALID_OPERATION => GLError::InvalidOperation,
            gl_raw::OUT_OF_MEMORY => GLError::OutOfMemory,
            gl_raw::INVALID_FRAMEBUFFER_OPERATION => GLError::InvalidFramebufferOperation,
            unknown_error => GLError::UnknownError(unknown_error),
        };

        Err(error)
    }
}

/// Return OpenGL version string.
pub fn get_version_string<'a>() -> &'a CStr {
    unsafe {
        let ptr_to_str = gl_raw::GetString(gl_raw::VERSION) as *const c_char;
        CStr::from_ptr(ptr_to_str)
    }
}

/// Return OpenGL vendor string.
pub fn get_vendor_string<'a>() -> &'a CStr {
    unsafe {
        let ptr_to_str = gl_raw::GetString(gl_raw::VENDOR) as *const c_char;
        CStr::from_ptr(ptr_to_str)
    }
}

/// Return OpenGL renderer string.
pub fn get_renderer_string<'a>() -> &'a CStr {
    unsafe {
        let ptr_to_str = gl_raw::GetString(gl_raw::RENDERER) as *const c_char;
        CStr::from_ptr(ptr_to_str)
    }
}