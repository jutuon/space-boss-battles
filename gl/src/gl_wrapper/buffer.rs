/*
gl/src/gl_wrapper/buffer.rs, 2017-07-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use super::gl_raw;
use self::gl_raw::types::*;

use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;


struct VertexBufferStatic {
    id: GLuint,
    vector_size: GLint,
}

impl VertexBufferStatic {
    unsafe fn new(data: &[f32], vector_size: GLint) -> VertexBufferStatic {

        let mut id: GLuint = 0;

        unsafe {
            gl_raw::GenBuffers(1, &mut id);
            gl_raw::BindBuffer(gl_raw::ARRAY_BUFFER, id);

            let size: GLsizeiptr = (size_of::<f32>() * data.len()) as GLsizeiptr;
            let data_ptr = data.as_ptr() as *const c_void;

            gl_raw::BufferData(gl_raw::ARRAY_BUFFER, size, data_ptr, gl_raw::STATIC_DRAW);

        }

        VertexBufferStatic {id, vector_size}
    }

    fn set_vertex_attributes(&mut self, attribute_index: GLuint, vertex_array: &VertexArray) {
        vertex_array.bind();

        unsafe {
            let stride = (self.vector_size * size_of::<f32>() as GLint) as GLsizei;
            gl_raw::VertexAttribPointer(attribute_index, self.vector_size, gl_raw::FLOAT, gl_raw::FALSE, stride, ptr::null());
            gl_raw::EnableVertexAttribArray(attribute_index);
        }
    }
}

impl Drop for VertexBufferStatic {
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteBuffers(1, &self.id);
        }
    }
}


pub struct VertexArray {
    id: GLuint,
    vertex_buffers: Vec<VertexBufferStatic>,
    vertex_count: GLsizei,
}

impl VertexArray {
    pub fn new(vertex_count: GLsizei) -> VertexArray {
        let mut id: GLuint = 0;
        let vertex_buffers = vec![];

        unsafe {
            gl_raw::GenVertexArrays(1, &mut id);
            VertexArray {id, vertex_buffers, vertex_count}
        }
    }

    pub fn add_static_buffer(&mut self, data: &[f32], vector_size: GLint, attribute_index: GLuint) {
        if data.len() / vector_size as usize != self.vertex_count as usize {
            panic!("buffer size doesn't match with vertex array's vertex count");
        }

        if data.len() % vector_size as usize != 0 {
            panic!("count of elements in data does not match vector size");
        }

        let mut buffer;

        unsafe {
            buffer = VertexBufferStatic::new(data, vector_size);
        }

        buffer.set_vertex_attributes(attribute_index, self);
        self.vertex_buffers.push(buffer);
    }

    fn bind(&self) {
        unsafe {
            gl_raw::BindVertexArray(self.id);
        }
    }

    pub fn draw(&self) {
        self.bind();

        unsafe {
            gl_raw::DrawArrays(gl_raw::TRIANGLES, 0, self.vertex_count);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl_raw::DeleteBuffers(1, &self.id);
        }
    }
}