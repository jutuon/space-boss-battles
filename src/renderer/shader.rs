/*
src/renderer/shader.rs, 2017-08-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Shaders for `OpenGLRenderer`.
//!
//! Note that vertex shader's vertex attribute
//! variable indexes will be set in function `create_program`.
//! See function's documentation for more details.

use std::fs::File;
use std::io::Read;
use std::ffi::CString;

use gl::shader::*;
use gl::uniform::*;

use cgmath::{Matrix4, Vector3};

/// Render with texture. Supports OpenGL 3.3 and OpenGL ES 2.0.
pub struct TextureShader {
    program: Program,
    projection: UniformMatrix4,
    model: UniformMatrix4,
}

impl TextureShader {
    /// Creates new TextureShader
    ///
    /// # Panics
    /// If there is some error in creating the shader or uniforms.
    pub fn new() -> TextureShader {

        #[cfg(feature = "gles")]
        let program = create_program("src/shaders/gles/vertex-shader-gles.glsl", "src/shaders/gles/fragment-shader-gles.glsl");

        #[cfg(not(feature = "gles"))]
        let program = create_program("src/shaders/gl/vertex-shader.glsl", "src/shaders/gl/fragment-shader.glsl");

        let model = create_uniform("M", &program, "texture shader");
        let projection = create_uniform("P", &program, "texture shader");

        TextureShader { program, projection, model }
    }

    /// Sends uniform data specific to this shader to GPU.
    pub fn send_uniform_data(&mut self, model: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.model.send(model);
        self.projection.send(projection);
    }

    /// Tell OpenGL to use this shader program.
    pub fn use_program(&mut self) {
        self.program.use_program();
    }
}

/// Render tile map tiles. Supports OpenGL 3.3 and OpenGL ES 2.0.
pub struct TileMapShader {
    program: Program,
    projection: UniformMatrix4,
    model: UniformMatrix4,
    tile_position_change_x_y_and_scaling_factor: UniformVector3,
}

impl TileMapShader {
    /// Creates new TileMapShader
    ///
    /// # Panics
    /// If there is some error in creating the shader or uniforms.
    pub fn new() -> TileMapShader {

        #[cfg(feature = "gles")]
        let program = create_program("src/shaders/gles/vertex-shader-tilemap-gles.glsl", "src/shaders/gles/fragment-shader-tilemap-gles.glsl");

        #[cfg(not(feature = "gles"))]
        let program = create_program("src/shaders/gl/vertex-shader-tilemap.glsl", "src/shaders/gl/fragment-shader-tilemap.glsl");

        let model = create_uniform("M", &program, "tilemap shader");
        let projection = create_uniform("P", &program, "tilemap shader");
        let tile_position_change_x_y_and_scaling_factor = create_uniform("tile_info", &program, "tilemap shader");

        TileMapShader { program, projection, model, tile_position_change_x_y_and_scaling_factor }
    }

    /// Sends uniform data specific to this shader to GPU.
    pub fn send_uniform_data(&mut self, model: &Matrix4<f32>, projection: &Matrix4<f32>, tile_position_change_x_y_and_scaling_factor: &Vector3<f32>) {
        self.model.send(model);
        self.projection.send(projection);
        self.tile_position_change_x_y_and_scaling_factor.send(tile_position_change_x_y_and_scaling_factor);
    }

    /// Tell OpenGL to use this shader program.
    pub fn use_program(&mut self) {
        self.program.use_program();
    }
}

/// Render with specific color. Supports OpenGL 3.3 and OpenGL ES 2.0.
pub struct ColorShader {
    program: Program,
    projection: UniformMatrix4,
    model: UniformMatrix4,
    color: UniformVector3,
}

impl ColorShader {
    /// Creates new ColorShader
    ///
    /// # Panics
    /// If there is some error in creating the shader or uniforms.
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

    /// Sends uniform data specific to this shader to GPU.
    pub fn send_uniform_data(&mut self, model: &Matrix4<f32>, projection: &Matrix4<f32>, color: &Vector3<f32>) {
        self.model.send(model);
        self.projection.send(projection);
        self.color.send(color);
    }

    /// Tell OpenGL to use this shader program.
    pub fn use_program(&mut self) {
        self.program.use_program();
    }
}

/// Build shader program from file paths.
///
/// # Panics
/// * If there is error opening or reading the files.
/// * There is error compiling or linking the shaders.
/// * Shader code contains 0 byte.
///
/// # Vertex attribute variable indexes
/// * variable "vertex", index 0
/// * variable "texture_coordinates_attribute", index 1
///
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

/// Read shader source code from file and create shader of type `ShaderType`.
///
/// # Panics
/// * If there is error opening or reading the files.
/// * There is error compiling the shader.
/// * Shader code contains 0 byte.
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

/// Create uniform specific to one shader program.
///
/// # Panics
/// * If `name` argument contains 0 byte.
/// * If there is not uniform with name that equals argument `name` in the shader program.
fn create_uniform<T: Uniform>(name: &str, program: &Program, program_name: &str) -> T {
    let uniform_result = T::new(CString::new(name).unwrap(), &program);

    match uniform_result {
        Ok(uniform) => uniform,
        Err(error) => {
            println!("error: {:?}\n uniform name: {}\n program name: {}\n", error, name, program_name);
            panic!();
        },
    }
}