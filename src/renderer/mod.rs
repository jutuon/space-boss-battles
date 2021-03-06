/*
src/renderer/mod.rs, 2017-09-01

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Render GUI and Logic.

mod texture;
mod shader;

use window::{Window, RenderingContext};

use cgmath::{Vector3, Matrix4, Point2, Vector4};
use cgmath;
use cgmath::prelude::*;

use gl::buffer::*;
use gl::texture::*;
use gl::gl_raw;
use gl;

use renderer::texture::Textures;
use renderer::shader::*;

use logic::{Logic, LaserColor};

use gui::GUI;
use gui::components::GUIText;

pub const DEFAULT_SCREEN_WIDTH: i32 = 640;
pub const DEFAULT_SCREEN_HEIGHT: i32 = 480;

const BLUE_COLOR: Vector3<f32> = Vector3 { x: 0.0, y: 0.0, z: 1.0 };
const RED_COLOR: Vector3<f32> = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
const GREEN_LASER_COLOR: Vector3<f32> = Vector3 { x: 0.0, y: 0.5, z: 0.0 };
const PARTICLE_COLOR: Vector3<f32> = Vector3 { x: 0.3, y: 0.3, z: 0.3 };

// FIXME: Changing this value makes GUI element positioning
//        and object movement limits not match screen size.
pub const SCREEN_TOP_Y_VALUE_IN_WORLD_COORDINATES: f32 = 4.5;

/// Model matrix for rendering.
pub trait ModelMatrix {
    /// Get model matrix.
    fn model_matrix(&self) -> &Matrix4<f32>;
}

/// Color for rendering.
pub trait Color {
    /// Get color.
    fn color(&self) -> &Vector3<f32>;
}

/// Render tile from tile map.
pub trait TileLocationInfo {
    /// Tile's location information.
    ///
    /// # Vector3 components
    /// vector[0]: movement in x direction.
    /// vector[1]: movement in y direction.
    /// vector[2]: scaling factor for texture coordinates.
    ///
    /// Shader will multiply texture coordinates with scaling factor
    /// and then add x and y movement to multiplied texture coordinates.
    ///
    /// For tile rendering, scaling factor should be less than 1.0.
    /// In practice, the scaling factor will make square represented by texture coordinates
    /// smaller, positioned at lower left corner of the texture. Then with x and y movement
    /// values you can move that square to a specific location on a tile map.
    fn tile_info(&self) -> &Vector3<f32>;
}

/// OpenGL 3.0 and OpenGL ES 2.0 renderer.
///
/// When compiling with feature "gles" you must only load
/// OpenGL ES 2.0 compatible shaders.
pub struct OpenGLRenderer {
    textures: [Texture; Textures::TextureCount as usize],
    texture_shader: TextureShader,
    color_shader: ColorShader,
    tile_map_shader: TileMapShader,
    /// Vertex and texture coordinates of square.
    square: VertexArray,
    projection_matrix: Matrix4<f32>,
    /// Go back to world coordinates from normalized device coordinates.
    inverse_projection_matrix: Matrix4<f32>,
    screen_width: i32,
    screen_height: i32,
    half_screen_width_world_coordinates: f32,
}

/// Interface for renderers.
///
/// This enables you to write different renderers without
/// changing other codes.
pub trait Renderer {
    /// Start rendering new frame. Call this first.
    fn start(&mut self);
    /// Render game logic.
    fn render(&mut self, &Logic, only_background: bool);
    /// Render GUI.
    fn render_gui(&mut self, &GUI);
    /// End rendering of new frame. Call this last.
    fn end<W: Window>(&mut self, &mut W);
    /// Converts screen coordinates to world coordinates.
    ///
    /// # Coordinates
    /// * Start form top left corner of the window.
    /// * Are in pixels.
    /// * Are at range [0, i32::MAX].
    // FIXME: use unsigned values for coordinates? SDL2 event makes i32 values.
    fn screen_coordinates_to_world_coordinates(&self, x: i32, y: i32) -> Point2<f32>;

    /// Screen width in world coordinates divided by 2.
    fn half_screen_width_world_coordinates(&self) -> f32;

    /// Update renderer to match new screen size.
    fn update_screen_size(&mut self, new_width_in_pixels: i32, new_height_in_pixels: i32);

    /// Get current screen width in pixels
    fn screen_width_pixels(&self) -> i32;
}

impl Renderer for OpenGLRenderer {

    /// Clears OpenGL color buffer.
    fn start(&mut self) {
        unsafe {
            gl_raw::Clear(gl_raw::COLOR_BUFFER_BIT);
        }
    }

    fn render(&mut self, logic: &Logic, only_background: bool) {
        self.texture_shader.use_program();

        self.textures[Textures::Background as usize].bind();
        for background in logic.get_moving_background().get_backgrounds() {
            self.render_rectangle_with_texture(background);
        }

        if only_background {
            return;
        }

        if logic.get_player().visible() {
            self.textures[Textures::Player as usize].bind();
            self.render_rectangle_with_texture(logic.get_player());
        }

        if logic.get_enemy().visible() {
            if logic.get_enemy().get_laser_cannon_top().visible() {
                self.textures[Textures::EnemyWithShield as usize].bind();
                self.render_rectangle_with_texture(logic.get_enemy());

                if logic.get_enemy().get_laser_cannon_top().red_light() {
                    self.textures[Textures::LaserCannonRed as usize].bind();
                } else {
                    self.textures[Textures::LaserCannonGreen as usize].bind();
                }
                self.render_rectangle_with_texture(logic.get_enemy().get_laser_cannon_top());
            } else {
                self.textures[Textures::Enemy as usize].bind();
                self.render_rectangle_with_texture(logic.get_enemy());
            }

            if logic.get_enemy().get_laser_cannon_bottom().visible() {
                if logic.get_enemy().get_laser_cannon_bottom().red_light() {
                    self.textures[Textures::LaserCannonRed as usize].bind();
                } else {
                    self.textures[Textures::LaserCannonGreen as usize].bind();
                }
                self.render_rectangle_with_texture(logic.get_enemy().get_laser_cannon_bottom());
            }

            if logic.get_enemy().get_shield().visible() {
                self.textures[Textures::Shield as usize].bind();
                self.render_rectangle_with_texture(logic.get_enemy().get_shield());
            }
        }

        for laser_bomb in logic.get_enemy().get_laser_bombs() {
            self.textures[Textures::LaserBomb as usize].bind();
            self.render_rectangle_with_texture(laser_bomb);
        }

        self.color_shader.use_program();

        for laser in logic.get_player().get_lasers() {
            self.render_color_rectangle_with_color(laser, &GREEN_LASER_COLOR);
        }

        for laser in logic.get_enemy().get_lasers() {
            if let LaserColor::Red = laser.color() {
                self.render_color_rectangle_with_color(laser, &RED_COLOR);
            } else {
                self.render_color_rectangle_with_color(laser, &BLUE_COLOR);
            }
        }

        if logic.get_explosion().visible() {
            for particle in logic.get_explosion().particles() {
                self.render_color_rectangle_with_color(particle, &PARTICLE_COLOR);
            }
        }
    }

    fn render_gui(&mut self, gui: &GUI) {
        let components = gui.components();

        self.color_shader.use_program();

        for button in components.buttons() {
            self.render_color_rectangle(button);
        }

        for health_bar in components.health_bars() {
            self.render_color_rectangle(health_bar);

            for border in health_bar.borders().into_iter() {
                self.render_color_rectangle_with_color(*border, health_bar.border_color());
            }
        }

        self.tile_map_shader.use_program();
        self.textures[Textures::Font as usize].bind();

        for text in components.texts() {
            self.render_text(text);
        }

        for button in components.buttons() {
            self.render_text(button.get_text());
        }

        if gui.get_gui_fps_counter().show_fps() {
            for text in gui.get_gui_fps_counter().texts().into_iter() {
                self.render_text(text);
            }
        }
    }

    /// Swap color buffers and check OpenGL errors.
    fn end<W: Window>(&mut self, window: &mut W) {
        window.swap_buffers().expect("couldn't swap rendering buffers");

        while let Err(error) = gl::GLError::get_error() {
            println!("OpenGL error: {:?}", error);
        }
    }

    /// Converts x and y to OpenGL normalized device coordinates [-1.0,1.0] and
    /// multiplies converted coordinates with `inverse_projection_matrix`.
    fn screen_coordinates_to_world_coordinates(&self, x: i32, y: i32) -> Point2<f32> {
        let width = self.screen_width/2;
        let height = self.screen_height/2;
        let x: f32 = (x - width) as f32 / width as f32;
        let y: f32 = (y - height) as f32 / -height as f32;

        let vector = self.inverse_projection_matrix * Vector4::new(x, y, 0.0, 1.0);

        Point2::new(vector.x,vector.y)
    }

    /// Updates fields `screen_width` and `screen_height`,
    /// OpenGL viewport, and projection matrix to match current screen size.
    fn update_screen_size(&mut self, new_width_in_pixels: i32, new_height_in_pixels: i32) {
        unsafe {
            gl_raw::Viewport(0,0,new_width_in_pixels, new_height_in_pixels);
        }

        self.screen_width = new_width_in_pixels;
        self.screen_height = new_height_in_pixels;

        self.update_projection_matrix();
    }

    fn half_screen_width_world_coordinates(&self) -> f32 {
        self.half_screen_width_world_coordinates
    }

    fn screen_width_pixels(&self) -> i32 {
        self.screen_width
    }
}

impl OpenGLRenderer {
    /// Creates new OpenGLRenderer.
    pub fn new<W: Window>(window: &W) -> OpenGLRenderer {
        gl_raw::load_with(|name| window.gl_get_proc_address(name));

        unsafe {
            gl_raw::ClearColor(0.0,0.0,0.0,1.0);
        }

        println!("OpenGL context information:");
        println!("  Version:  {:?}", gl::get_version_string());
        println!("  Vendor:   {:?}", gl::get_vendor_string());
        println!("  Renderer: {:?}", gl::get_renderer_string());

        let mut renderer = OpenGLRenderer {
            texture_shader: TextureShader::new(),
            color_shader: ColorShader::new(),
            tile_map_shader: TileMapShader::new(),
            textures: Textures::load_all(),
            square: create_square(),
            projection_matrix: Matrix4::identity(),
            inverse_projection_matrix: Matrix4::identity(),
            screen_width: DEFAULT_SCREEN_WIDTH,
            screen_height: DEFAULT_SCREEN_HEIGHT,
            half_screen_width_world_coordinates: 1.0,
        };

        // Update fields projection_matrix, inverse_projection_matrix
        // and half_screen_width_world_coordinates to have correct value.
        renderer.update_screen_size(DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);

        renderer
    }

    /// Updates `OpenGLRenderer` fields `half_screen_width_world_coordinates`,
    /// `projection_matrix` and `inverse_projection_matrix` from fields `screen_width` and `screen_height`
    ///
    /// # Errors
    /// If inverse matrix calculation fails `inverse_projection_matrix` field will be set to identity matrix.
    fn update_projection_matrix(&mut self) {
        self.half_screen_width_world_coordinates = (self.screen_width as f32 /self.screen_height as f32) * SCREEN_TOP_Y_VALUE_IN_WORLD_COORDINATES;
        self.projection_matrix = cgmath::ortho::<f32>(-self.half_screen_width_world_coordinates, self.half_screen_width_world_coordinates, -SCREEN_TOP_Y_VALUE_IN_WORLD_COORDINATES, SCREEN_TOP_Y_VALUE_IN_WORLD_COORDINATES, 1.0, -1.0);

        match self.projection_matrix.inverse_transform() {
            Some(matrix) => self.inverse_projection_matrix = matrix,
            None => {
                println!("Calculating inverse projection matrix failed");
                self.inverse_projection_matrix = Matrix4::identity();
            }
        };
    }

    /// Render `GUIText`. Bind correct texture before calling this method.
    fn render_text(&mut self, text: &GUIText) {
        for tile in text.get_tiles() {
            self.render_tile(tile);
        }
    }

    /// Render tile. Bind correct texture before calling this method.
    fn render_tile<T: ModelMatrix + TileLocationInfo>(&mut self, tile: &T) {
        self.tile_map_shader.send_uniform_data(tile.model_matrix(), &self.projection_matrix, tile.tile_info());
        self.square.draw();
    }

    /// Render rectangle with object specified color.
    fn render_color_rectangle<T: ModelMatrix + Color>(&mut self, object: &T) {
        self.color_shader.send_uniform_data(object.model_matrix(), &self.projection_matrix, object.color());
        self.square.draw();
    }

    /// Render rectangle with color from argument.
    fn render_color_rectangle_with_color<T: ModelMatrix>(&mut self, object: &T, color: &Vector3<f32>) {
        self.color_shader.send_uniform_data(object.model_matrix(), &self.projection_matrix, color);
        self.square.draw();
    }

    /// Render rectangle with texture. Bind correct texture before calling this method.
    fn render_rectangle_with_texture<T: ModelMatrix>(&mut self, object: &T) {
        self.texture_shader.send_uniform_data(object.model_matrix(), &self.projection_matrix);
        self.square.draw();
    }
}

/// Create `VertexArray` with vertex and texture
/// coordinate data of square.
///
/// Vertex data will be set to attribute index 0 and
/// texture data will be set to attribute index 1 when rendering
/// with this vertex array.
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