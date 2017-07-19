/*
gl/src/gl_wrapper/buffer.rs, 2017-07-19

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Send data to GPU.

use super::gl_raw;
use self::gl_raw::types::*;

use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;


/// Send static data to GPU with Vertex Buffer Object
struct VertexBufferStatic {
    id: GLuint,
    attribute_component_count: GLint,
}

impl VertexBufferStatic {
    /// Sends static data to GPU.
    ///
    /// # Arguments
    /// * `data` - Float data which is sent to GPU.
    /// * `attribute_component_count` - Number of floats in one vertex attribute.
    ///
    /// # Safety
    /// This function does not check if data length and `attribute_component_count` match.
    unsafe fn new(data: &[f32], attribute_component_count: GLint) -> VertexBufferStatic {
        let mut id: GLuint = 0;

        gl_raw::GenBuffers(1, &mut id);
        gl_raw::BindBuffer(gl_raw::ARRAY_BUFFER, id);

        let size: GLsizeiptr = (size_of::<f32>() * data.len()) as GLsizeiptr;
        let data_ptr = data.as_ptr() as *const c_void;

        gl_raw::BufferData(gl_raw::ARRAY_BUFFER, size, data_ptr, gl_raw::STATIC_DRAW);

        VertexBufferStatic {id, attribute_component_count}
    }

    /// Set vertex attribute to match buffer data.
    ///
    /// # Arguments
    /// * `attribute_index` - Index of vertex attribute.
    fn set_vertex_attributes(&mut self, attribute_index: GLuint) {
        unsafe {
            gl_raw::BindBuffer(gl_raw::ARRAY_BUFFER, self.id);

            let stride = (self.attribute_component_count * size_of::<f32>() as GLint) as GLsizei;
            gl_raw::VertexAttribPointer(attribute_index, self.attribute_component_count, gl_raw::FLOAT, gl_raw::FALSE, stride, ptr::null());
            gl_raw::EnableVertexAttribArray(attribute_index);
        }
    }
}

impl Drop for VertexBufferStatic {

    /// Deletes OpenGL's buffer object.
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteBuffers(1, &self.id);
        }
    }
}

/// Send multiple buffers of data to GPU.
///
/// OpenGL 3.3 version of this struct is implemented
/// with OpenGL's Vertex Array Object.
///
/// OpenGL ES 2.0 does not support Vertex Array Objects, so vertex
/// attributes are set for every buffer
/// when `draw` method is called if using OpenGL ES version of this struct.
#[cfg(not(feature = "gles"))]
pub struct VertexArray {
    id: GLuint,
    vertex_buffers: Vec<VertexBufferStatic>,
    vertex_count: GLsizei,
}

#[cfg(feature = "gles")]
pub struct VertexArray {
    vertex_buffers: Vec<(VertexBufferStatic, GLuint)>,
    vertex_count: GLsizei,
}


impl VertexArray {

    /// Creates new Vertex Array Object
    #[cfg(not(feature = "gles"))]
    pub fn new(vertex_count: GLsizei) -> VertexArray {
        let mut id: GLuint = 0;
        let vertex_buffers = vec![];

        unsafe {
            gl_raw::GenVertexArrays(1, &mut id);
            VertexArray {id, vertex_buffers, vertex_count}
        }
    }

    #[cfg(feature = "gles")]
    pub fn new(vertex_count: GLsizei) -> VertexArray {
        let vertex_buffers = vec![];
        VertexArray {vertex_buffers, vertex_count}
    }

    /// Adds new buffer to Vertex Array Object
    ///
    /// # Arguments
    /// * `data` - Float data to send to the GPU.
    /// * `attribute_component_count` - Number of floats in one attribute.
    /// * `attribute_index` - Index of vertex attribute.
    ///
    /// # Panics
    /// * If buffer length doesn't match with `VertexArray`'s vertex count.
    /// * If buffer length doesn't match with attribute_component_count.
    pub fn add_static_buffer(&mut self, data: &[f32], attribute_component_count: GLint, attribute_index: GLuint) {
        if data.len() / attribute_component_count as usize != self.vertex_count as usize {
            panic!("buffer length doesn't match with VertexArray's vertex count");
        }

        if data.len() % attribute_component_count as usize != 0 {
            panic!("buffer length doesn't match with attribute_component_count");
        }

        #[cfg(not(feature = "gles"))]
        {
            let mut buffer;

            unsafe {
                buffer = VertexBufferStatic::new(data, attribute_component_count);
            }

            self.bind();
            buffer.set_vertex_attributes(attribute_index);
            self.vertex_buffers.push(buffer);
        }

        #[cfg(feature = "gles")]
        {
            let buffer;

            unsafe {
                buffer = VertexBufferStatic::new(data, attribute_component_count);
            }

            self.vertex_buffers.push((buffer, attribute_index));
        }
    }

    /// Bind OpenGL's Vertex Array Object. This method
    /// only exists for OpenGL 3.3 version of `VertexArray` struct.
    #[cfg(not(feature = "gles"))]
    fn bind(&self) {
        unsafe {
            gl_raw::BindVertexArray(self.id);
        }
    }

    /// Draw with buffers currently existing buffers in `VertexArray`. Remember to enable
    /// correct shader `Program` with it's `use_program` method before calling this method.
    pub fn draw(&mut self) {
        #[cfg(not(feature = "gles"))]
        {
            self.bind();
        }

        #[cfg(feature = "gles")]
        {
            for &mut (ref mut buffer, attribute_index) in &mut self.vertex_buffers {
                buffer.set_vertex_attributes(attribute_index);
            }
        }

        unsafe {
            gl_raw::DrawArrays(gl_raw::TRIANGLES, 0, self.vertex_count);
        }
    }
}

#[cfg(not(feature = "gles"))]
impl Drop for VertexArray {
    /// Delete OpenGL's Vertex Array Object. This implementation of Drop trait
    /// only exists for OpenGL 3.3 version of `VertexArray` struct.
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteVertexArrays(1, &self.id);
        }
    }
}