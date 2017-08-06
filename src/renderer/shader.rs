/*
src/renderer/shader.rs, 2017-08-02

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use std::fs::File;
use std::io::Read;
use std::ffi::CString;

use gl::shader::*;
use gl::uniform::*;

use cgmath::{Matrix4, Vector3};

pub struct TextureShader {
    program: Program,
    projection: UniformMatrix4,
    model: UniformMatrix4,
}

impl TextureShader {
    pub fn new() -> TextureShader {

        #[cfg(feature = "gles")]
        let program = create_program("src/shaders/gles/vertex-shader-gles.glsl", "src/shaders/gles/fragment-shader-gles.glsl");

        #[cfg(not(feature = "gles"))]
        let program = create_program("src/shaders/gl/vertex-shader.glsl", "src/shaders/gl/fragment-shader.glsl");

        let model = create_uniform("M", &program, "texture shader");
        let projection = create_uniform("P", &program, "texture shader");

        TextureShader { program, projection, model }
    }

    pub fn send_uniform_data(&mut self, model: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.model.send(model);
        self.projection.send(projection);
    }

    pub fn use_program(&mut self) {
        self.program.use_program();
    }
}

pub struct TilemapShader {
    program: Program,
    projection: UniformMatrix4,
    model: UniformMatrix4,
    tile_position_change_x_y_and_size: UniformVector3,
}

impl TilemapShader {
    pub fn new() -> TilemapShader {

        #[cfg(feature = "gles")]
        let program = create_program("src/shaders/gles/vertex-shader-tilemap-gles.glsl", "src/shaders/gles/fragment-shader-tilemap-gles.glsl");

        #[cfg(not(feature = "gles"))]
        let program = create_program("src/shaders/gl/vertex-shader-tilemap.glsl", "src/shaders/gl/fragment-shader-tilemap.glsl");

        let model = create_uniform("M", &program, "tilemap shader");
        let projection = create_uniform("P", &program, "tilemap shader");
        let tile_position_change_x_y_and_size = create_uniform("tile_info", &program, "tilemap shader");

        TilemapShader { program, projection, model, tile_position_change_x_y_and_size }
    }

    pub fn send_uniform_data(&mut self, model: &Matrix4<f32>, projection: &Matrix4<f32>, tile_position_change_x_y_and_size: &Vector3<f32>) {
        self.model.send(model);
        self.projection.send(projection);
        self.tile_position_change_x_y_and_size.send(tile_position_change_x_y_and_size);
    }

    pub fn use_program(&mut self) {
        self.program.use_program();
    }
}


pub struct ColorShader {
    program: Program,
    projection: UniformMatrix4,
    model: UniformMatrix4,
    color: UniformVector3,

}

impl ColorShader {
    pub fn new() -> ColorShader {

        #[cfg(feature = "gles")]
        let program = create_program("src/shaders/gles/color-vertex-gles.glsl", "src/shaders/gles/color-fragment-gles.glsl");

        #[cfg(not(feature = "gles"))]
        let program = create_program("src/shaders/gl/color-vertex.glsl", "src/shaders/gl/color-fragment.glsl");

        let model = create_uniform("M", &program, "color shader");
        let projection = create_uniform("P", &program, "color shader");
        let color = create_uniform("color", &program, "color shader");

        ColorShader { program, projection, model, color }
    }

    pub fn send_uniform_data(&mut self, model: &Matrix4<f32>, projection: &Matrix4<f32>, color: &Vector3<f32>) {
        self.model.send(model);
        self.projection.send(projection);
        self.color.send(color);
    }

    pub fn use_program(&mut self) {
        self.program.use_program();
    }
}

fn create_program(vertex_shader_path: &str, fragment_shader_path: &str) -> Program {
    let vertex_shader = load_shader(ShaderType::Vertex, vertex_shader_path);
    let fragment_shader = load_shader(ShaderType::Fragment, fragment_shader_path);

    let mut vertex_attributes = VertexAttributeIndexBinder::new();
    vertex_attributes.add_attribute(0, "vertex");
    vertex_attributes.add_attribute(1, "texture_coordinates_attribute");

    match Program::new(vertex_shader, fragment_shader, vertex_attributes) {
        Ok(program) => program,
        Err(message) => {
            println!("program creation error:\n{}", message);
            panic!();
        }
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

fn create_uniform<T: Uniform>(name: &str, program: &Program, program_name: &str) -> T {
    let uniform_result = T::new(CString::new(name).unwrap(), &program);

    handle_uniform_error(name, program_name, uniform_result)
}

fn handle_uniform_error<T>(name: &str, program_name: &str, uniform_result: Result<T, UniformError>) -> T {
    match uniform_result {
        Ok(uniform) => uniform,
        Err(error) => {
            println!("error: {:?}\n uniform name: {}\n program name: {}\n", error, name, program_name);
            panic!();
        },
    }
}