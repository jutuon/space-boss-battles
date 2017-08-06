/*
src/renderer/mod.rs, 2017-08-06

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

use cgmath::{Vector3, Matrix4, Point2, Vector4};
use cgmath;
use cgmath::prelude::*;

use renderer::texture::Textures;
use renderer::shader::*;

use sdl2::VideoSubsystem;
use sdl2::video::{Window, FullscreenType};
use sdl2::video::{GLProfile, GLContext};

use logic::{Logic};
use logic::common::ModelMatrix;

use gui::{GUI, GUILayerComponents};

pub struct OpenGLRenderer {
    video_system: VideoSubsystem,
    window: Window,
    context: GLContext,
    textures: [Texture; Textures::TextureCount as usize],
    texture_shader: TextureShader,
    color_shader: ColorShader,
    tilemap_shader: TilemapShader,
    square: VertexArray,
    projection_matrix: Matrix4<f32>,
    inverse_projection_matrix: Matrix4<f32>,
    screen_width: i32,
    screen_height: i32,
    half_screen_width_world_coordinates: f32,
}

pub trait Renderer {
    fn start(&mut self);
    fn render(&mut self, &Logic);
    fn render_gui(&mut self, &GUI);
    fn end(&mut self);
    fn screen_coordinates_to_world_coordinates(&self, x: i32, y: i32) -> Point2<f32>;
    fn full_screen(&mut self, value: bool);
    fn v_sync(&mut self, value: bool);
    fn half_screen_width_world_coordinates(&self) -> f32;
}

impl Renderer for OpenGLRenderer {

    fn start(&mut self) {
        unsafe {
            gl_raw::Clear(gl_raw::COLOR_BUFFER_BIT);
        }
    }

    fn render(&mut self, logic: &Logic) {
        self.texture_shader.use_program();

        self.textures[Textures::Background as usize].bind();
        for background in logic.get_moving_background().get_backgrounds() {
            self.texture_shader.send_uniform_data(background.model_matrix(), &self.projection_matrix);
            self.square.draw();
        }

        self.textures[Textures::Player as usize].bind();
        self.texture_shader.send_uniform_data(logic.get_player().model_matrix(), &self.projection_matrix);
        self.square.draw();

        self.textures[Textures::Enemy as usize].bind();
        self.texture_shader.send_uniform_data(logic.get_enemy().model_matrix(), &self.projection_matrix);
        self.square.draw();

        self.color_shader.use_program();

        let color = Vector3::new(0.0,0.0,1.0);
        for laser in logic.get_player().get_lasers() {
            self.color_shader.send_uniform_data(laser.model_matrix(), &self.projection_matrix, &color);
            self.square.draw();
        }

        let color = Vector3::new(1.0,0.0,0.0);
        for laser in logic.get_enemy().get_lasers() {
            self.color_shader.send_uniform_data(laser.model_matrix(), &self.projection_matrix, &color);
            self.square.draw();
        }
    }

    fn render_gui(&mut self, gui: &GUI) {

        let (buttons, texts) = gui.components();

        self.color_shader.use_program();

        for button in buttons {
            self.color_shader.send_uniform_data(button.model_matrix(), &self.projection_matrix, button.color());
            self.square.draw();
        }

        self.tilemap_shader.use_program();
        self.textures[Textures::Font as usize].bind();

        for text in texts {
            for tile in text.get_tiles() {
                self.tilemap_shader.send_uniform_data(tile.get_rectangle().model_matrix(), &self.projection_matrix, tile.get_tile_info());
                self.square.draw();
            }
        }

        for button in buttons {
            for tile in button.get_text().get_tiles() {
                self.tilemap_shader.send_uniform_data(tile.get_rectangle().model_matrix(), &self.projection_matrix, tile.get_tile_info());
                self.square.draw();
            }
        }

        if gui.get_gui_fps_counter().show_fps() {
            for text in gui.get_gui_fps_counter().texts().into_iter() {
                for tile in text.get_tiles() {
                    self.tilemap_shader.send_uniform_data(tile.get_rectangle().model_matrix(), &self.projection_matrix, tile.get_tile_info());
                    self.square.draw();
                }
            }
        }
    }

    fn end(&mut self) {
        self.window.gl_swap_window();

        while let Err(error) = gl::GLError::get_error() {
            println!("OpenGL error: {:?}", error);
        }
    }

    fn screen_coordinates_to_world_coordinates(&self, x: i32, y: i32) -> Point2<f32> {
        let width = self.screen_width/2;
        let height = self.screen_height/2;
        let x: f32 = (x - width) as f32 / width as f32;
        let y: f32 = (y - height) as f32 / -height as f32;

        let vector = self.inverse_projection_matrix * Vector4::new(x, y, 0.0, 1.0);

        Point2::new(vector.x,vector.y)
    }

    fn full_screen(&mut self, value: bool) {
        let setting;

        if value {
            setting = FullscreenType::Desktop;
        } else {
            setting = FullscreenType::Off;
        }

        if let Err(message) = self.window.set_fullscreen(setting) {
            println!("Error, couldn't change fullscreen setting: {}", message);
        } else {
            self.update_screen_size();
            self.update_projection_matrix();
        }
    }

    fn v_sync(&mut self, value: bool) {
        if value {
            self.video_system.gl_set_swap_interval(1);
        } else {
            self.video_system.gl_set_swap_interval(0);
        }
    }

    fn half_screen_width_world_coordinates(&self) -> f32 {
        self.half_screen_width_world_coordinates
    }
}

impl OpenGLRenderer {
    pub fn new(video_system: VideoSubsystem) -> OpenGLRenderer {
        let screen_width = 640;
        let screen_height = 480;

        let window = video_system.window("Space Boss Battles", screen_width as u32, screen_height as u32).opengl().build().expect("window creation failed");

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
        let tilemap_shader = TilemapShader::new();

        unsafe {
            gl_raw::ClearColor(0.0,0.0,0.0,1.0);
        }

        let textures = Textures::load_all();
        let square = create_square();

        println!("OpenGL version: {:?}", gl::get_version_string());

        let projection_matrix = Matrix4::identity();
        let inverse_projection_matrix = Matrix4::identity();
        let half_screen_width_world_coordinates = 1.0;

        let mut renderer = OpenGLRenderer {video_system, window, context, texture_shader, color_shader, tilemap_shader, textures, square, projection_matrix, inverse_projection_matrix, screen_width, screen_height, half_screen_width_world_coordinates};
        renderer.update_screen_size();
        renderer.update_projection_matrix();

        renderer
    }

    fn update_projection_matrix(&mut self) {
        let size = 4.5;
        self.half_screen_width_world_coordinates = (self.screen_width as f32 /self.screen_height as f32) * size;
        let height = 1.0 * size;
        self.projection_matrix = cgmath::ortho::<f32>(-self.half_screen_width_world_coordinates, self.half_screen_width_world_coordinates, -height, height, 1.0, -1.0);

        match self.projection_matrix.inverse_transform() {
            Some(matrix) => self.inverse_projection_matrix = matrix,
            None => {
                println!("Calculating inverse projection matrix failed");
                self.inverse_projection_matrix = Matrix4::identity();
            }
        };
    }

    fn update_screen_size(&mut self) {
        let mut width = 640;
        let mut height = 480;

        match self.window.display_mode() {
            Ok(display_mode) => {
                width = display_mode.w;
                height = display_mode.h;
            },
            Err(message) => println!("couldn't get display mode info: {}", message),
        }

        unsafe {
            gl_raw::Viewport(0,0,width,height);
        }

        self.screen_width = width;
        self.screen_height = height;
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