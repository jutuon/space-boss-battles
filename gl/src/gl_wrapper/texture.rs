/*
gl/src/gl_wrapper/texture.rs, 2017-07-18

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Send textures to GPU.

use super::gl_raw;
use self::gl_raw::types::*;

use std::os::raw::c_void;

/// Texture with RGBA color
pub struct TextureRGBA {
    id: GLuint,
}

impl TextureRGBA {
    /// Send RGBA texture to GPU. This function will also set
    /// * Texture wrap mode to repeat.
    /// * Texture filtering to nearest.
    /// * Generate mipmap from the texture.
    ///
    /// # Panics
    /// If texture width and height does not match with data length
    /// this function will panic.
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> TextureRGBA {
        if width*height*4 != data.len() as u32 {
            panic!("image width and height does not match with data length");
        }

        let mut id: GLuint = 0;

        unsafe {
            gl_raw::GenTextures(1, &mut id);
        }

        let mut texture = TextureRGBA {id};
        texture.bind();

        unsafe {
            gl_raw::TexParameteri(gl_raw::TEXTURE_2D, gl_raw::TEXTURE_WRAP_S, gl_raw::REPEAT as GLint);
            gl_raw::TexParameteri(gl_raw::TEXTURE_2D, gl_raw::TEXTURE_WRAP_T, gl_raw::REPEAT as GLint);
            gl_raw::TexParameteri(gl_raw::TEXTURE_2D, gl_raw::TEXTURE_MIN_FILTER, gl_raw::NEAREST as GLint);
            gl_raw::TexParameteri(gl_raw::TEXTURE_2D, gl_raw::TEXTURE_MAG_FILTER, gl_raw::NEAREST as GLint);

            gl_raw::TexImage2D(gl_raw::TEXTURE_2D, 0, gl_raw::RGBA as GLint, width as GLsizei, height as GLsizei, 0, gl_raw::RGBA, gl_raw::UNSIGNED_BYTE, data.as_ptr() as *const c_void);
            gl_raw::GenerateMipmap(gl_raw::TEXTURE_2D);
        }

        texture
    }

    /// Binds texture for rendering.
    pub fn bind(&mut self) {
        unsafe {
            gl_raw::BindTexture(gl_raw::TEXTURE_2D, self.id);
        }
    }
}

impl Drop for TextureRGBA {
    /// Deletes OpenGL texture object.
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteTextures(1, &self.id);
        }
    }
}