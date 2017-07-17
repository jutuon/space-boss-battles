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

use cgmath::{Matrix4, Vector4};

pub struct TextureShader {
    program: Program,
    projection_matrix_uniform: UniformMatrix4,
    model_matrix_uniform: UniformMatrix4,
}

impl TextureShader {
    pub fn new() -> TextureShader {
        let program = create_program("src/shaders/vertex-shader.glsl", "src/shaders/fragment-shader.glsl");

        let model_matrix_uniform = UniformMatrix4::new(CString::new("M").unwrap(), &program).expect("uniform error");
        let projection_matrix_uniform = UniformMatrix4::new(CString::new("P").unwrap(), &program).expect("uniform error");

        TextureShader { program, projection_matrix_uniform, model_matrix_uniform }
    }

    pub fn send_uniform_data(&mut self, model: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.model_matrix_uniform.send(model);
        self.projection_matrix_uniform.send(projection);
    }

    pub fn use_program(&mut self) {
        self.program.use_program();
    }
}

pub struct ColorShader {
    program: Program,
}

impl ColorShader {
    pub fn new() -> ColorShader {
        let program = create_program("src/shaders/color-vertex.glsl", "src/shaders/color-fragment.glsl");

        ColorShader { program }
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