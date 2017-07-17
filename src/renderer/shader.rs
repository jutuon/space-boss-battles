/*
src/renderer/shader.rs, 2017-07-17

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
        let program = create_program("src/shaders/vertex-shader.glsl", "src/shaders/fragment-shader.glsl");

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

pub struct ColorShader {
    program: Program,
    projection: UniformMatrix4,
    model: UniformMatrix4,
    color: UniformVector3,

}

impl ColorShader {
    pub fn new() -> ColorShader {
        let program = create_program("src/shaders/color-vertex.glsl", "src/shaders/color-fragment.glsl");

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

    match Program::new(vertex_shader, fragment_shader) {
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

fn create_uniform<T: CreateUniform>(name: &str, program: &Program, program_name: &str) -> T {
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