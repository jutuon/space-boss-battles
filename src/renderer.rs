/*
src/renderer.rs, 2017-07-14

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/


use sdl2::VideoSubsystem;
use sdl2::video::{Window};
use sdl2::video::{GLProfile, GLContext};


use cgmath::{Vector3, Matrix4};
use cgmath::prelude::*;
use cgmath;

use logic::Logic;

use gl::buffer::*;
use gl::shader::*;
use gl::uniform::*;
use gl::texture::*;

use gl::gl_raw;

use std::fs::File;
use std::io::Read;
use std::ffi::CString;

use image::png::PNGDecoder;
use image::{ImageDecoder, DecodingResult, ColorType};

pub enum Textures {
    Player,
    TextureCount,
}

impl Textures {
    fn load_all() -> [TextureRGBA; Textures::TextureCount as usize] {
        [Textures::load("game_files/images/player.png")]
    }

    fn load(file_path: &str) -> TextureRGBA {
        let img_file = File::open(file_path).expect("img opening fail");
        let mut img = PNGDecoder::new(img_file);

        let (width, height) = img.dimensions().expect("img dimensions fail");

        match img.colortype().expect("img color type fail") {
            ColorType::RGBA(_) => (),
            _ => panic!("image's color type is not RGBA"),
        }

        let img_data_result = img.read_image().expect("img decoding fail");

        let img_data = match img_data_result {
            DecodingResult::U8(data) => data,
            _ => panic!("unknown image data"),
        };

        TextureRGBA::new(width, height, img_data)
    }
}


pub struct OpenGLRenderer {
    video_system: VideoSubsystem,
    window: Window,
    context: GLContext,
    program: Program,
    textures: [TextureRGBA; Textures::TextureCount as usize],
    rectangle: VertexArray,
    projection_matrix_uniform: UniformMatrix4,
    model_matrix_uniform: UniformMatrix4,
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
        self.program.use_program();

        self.textures[Textures::Player as usize].bind();

        let size = 3.0;
        let width = 4.0 * size;
        let height = 3.0 * size;

        let projection_matrix = cgmath::ortho::<f32>(-width, width, -height, height, 1.0, -1.0);
        self.projection_matrix_uniform.send(&projection_matrix);

        let model_matrix = Matrix4::identity();
        self.model_matrix_uniform.send(&model_matrix);

        self.rectangle.draw();
    }

    fn end(&mut self) {
        self.window.gl_swap_window();
    }
}

impl OpenGLRenderer {
    pub fn new(video_system: VideoSubsystem) -> OpenGLRenderer {
        let window = video_system.window("Space Boss Battles", 640,480).opengl().build().expect("window creation failed");

        {
            let gl_attr = video_system.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_version(3,3);
        }

        let context = window.gl_create_context().expect("opengl context creation failed");
        gl_raw::load_with(|name| video_system.gl_get_proc_address(name) as *const _);

        window.gl_make_current(&context).expect("couldn't set opengl context to current thread");

        let vertex_shader = load_shader(ShaderType::Vertex,"src/shaders/vertex-shader.glsl");
        let fragment_shader = load_shader(ShaderType::Fragment,"src/shaders/fragment-shader.glsl");

        let program = match Program::new(vertex_shader, fragment_shader) {
            Ok(program) => program,
            Err(message) => {
                println!("program creation error:\n{}", message);
                panic!();
            }
        };

        program.use_program();

        //video_system.gl_set_swap_interval(0);

        unsafe {
            gl_raw::Viewport(0,0,640,480);
            gl_raw::ClearColor(0.0,0.0,0.0,1.0);
        }

        let textures = Textures::load_all();

        let mut rectangle = VertexArray::new(6);

        let size : f32 = 1.0;

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

        rectangle.add_static_buffer(&vertex_data, 3, 0);
        rectangle.add_static_buffer(&texture_coordinates_data, 2, 1);

        let model_matrix_uniform = UniformMatrix4::new(CString::new("M").unwrap(), &program).expect("uniform error");
        let projection_matrix_uniform = UniformMatrix4::new(CString::new("P").unwrap(), &program).expect("uniform error");

        OpenGLRenderer {video_system, window, context, program, textures, rectangle, projection_matrix_uniform, model_matrix_uniform}
    }

}

fn load_shader(shader_type: ShaderType, file_path: &str) -> Shader {
    let mut shader_file = File::open(file_path).expect("shader file not found");
    let mut shader_text = String::new();

    shader_file.read_to_string(&mut shader_text).unwrap();

    let shader_text = CString::new(shader_text).unwrap();

    match Shader::new(shader_type, shader_text) {
        Ok(shader) => shader,
        Err(message) => {
            println!("shader compile error\n{}", message);
            panic!();
        },
    }
}