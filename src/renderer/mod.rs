/*
src/renderer/mod.rs, 2017-07-20

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

mod texture;
mod shader;

use gl::buffer::*;
use gl::texture::*;

use gl::gl_raw;
use gl;

use cgmath::{Vector3, Matrix4};
use cgmath;

use renderer::texture::Textures;
use renderer::shader::*;

use sdl2::VideoSubsystem;
use sdl2::video::{Window};
use sdl2::video::{GLProfile, GLContext};

use logic::{Logic};
use logic::common::ModelMatrix;


pub struct OpenGLRenderer {
    video_system: VideoSubsystem,
    window: Window,
    context: GLContext,
    textures: [TextureRGBA; Textures::TextureCount as usize],
    texture_shader: TextureShader,
    color_shader: ColorShader,
    square: VertexArray,
}

pub trait Renderer {
    fn start(&mut self);
    fn render(&mut self, &Logic);
    fn end(&mut self);
}

impl Renderer for OpenGLRenderer {

    fn start(&mut self) {
        unsafe {
            gl_raw::Clear(gl_raw::COLOR_BUFFER_BIT);
        }
    }

    fn render(&mut self, logic: &Logic) {
        self.texture_shader.use_program();

        self.textures[Textures::Player as usize].bind();

        let size = 1.5;
        let width = 4.0 * size;
        let height = 3.0 * size;
        let projection_matrix = cgmath::ortho::<f32>(-width, width, -height, height, 1.0, -1.0);

        self.texture_shader.send_uniform_data(logic.get_player().model_matrix(), &projection_matrix);

        self.square.draw();

        self.color_shader.use_program();

        let color = Vector3::new(0.0,0.0,1.0);
        for laser in logic.get_player().get_lasers() {
            self.color_shader.send_uniform_data(laser.model_matrix(), &projection_matrix, &color);
            self.square.draw();
        }
    }

    fn end(&mut self) {
        self.window.gl_swap_window();

        while let Err(error) = gl::GLError::get_error() {
            println!("OpenGL error: {:?}", error);
        }
    }
}

impl OpenGLRenderer {
    pub fn new(video_system: VideoSubsystem) -> OpenGLRenderer {
        let window = video_system.window("Space Boss Battles", 640,480).opengl().build().expect("window creation failed");


        #[cfg(feature = "gles")]
        {
            let gl_attr = video_system.gl_attr();
            gl_attr.set_context_profile(GLProfile::GLES);
            gl_attr.set_context_version(2,0);
        }

        #[cfg(not(feature = "gles"))]
        {
            let gl_attr = video_system.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_version(3,3);
        }

        let context = window.gl_create_context().expect("opengl context creation failed");
        gl_raw::load_with(|name| video_system.gl_get_proc_address(name) as *const _);

        window.gl_make_current(&context).expect("couldn't set opengl context to current thread");

        let texture_shader = TextureShader::new();
        let color_shader = ColorShader::new();

        //video_system.gl_set_swap_interval(0);

        unsafe {
            gl_raw::Viewport(0,0,640,480);
            gl_raw::ClearColor(0.0,0.0,0.0,1.0);
        }

        let textures = Textures::load_all();
        let square = create_square();

        println!("OpenGL version: {:?}", gl::get_version_string());

        OpenGLRenderer {video_system, window, context, texture_shader, color_shader, textures, square}
    }

}

fn create_square() -> VertexArray {
    let mut square = VertexArray::new(6);

    let size : f32 = 0.5;

    let vertex_data: [f32; 18]  = [
                size, -size, 0.0,
                size, size, 0.0,
                -size, size, 0.0,

                size, -size, 0.0,
                -size, size, 0.0,
                -size, -size, 0.0,
    ];
    let texture_coordinates_data: [f32; 12]  = [
                1.0, 0.0,
                1.0, 1.0,
                0.0, 1.0,

                1.0, 0.0,
                0.0, 1.0,
                0.0, 0.0,
    ];

    square.add_static_buffer(&vertex_data, 3, 0);
    square.add_static_buffer(&texture_coordinates_data, 2, 1);

    square
}