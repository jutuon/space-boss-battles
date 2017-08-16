/*
src/renderer/mod.rs, 2017-08-16

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

mod texture;
mod shader;

use sdl2::VideoSubsystem;
use sdl2::video::{Window, FullscreenType, GLProfile, GLContext};

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

const DEFAULT_SCREEN_WIDTH: i32 = 640;
const DEFAULT_SCREEN_HEIGHT: i32 = 480;

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
    video_system: VideoSubsystem,
    window: Window,
    /// OpenGL context is stored here because it
    /// would be otherwise dropped.
    _context: GLContext,
    textures: [Texture; Textures::TextureCount as usize],
    texture_shader: TextureShader,
    color_shader: ColorShader,
    tile_map_shader: TileMapShader,
    /// Vertex and texture coordinates of square.
    square: VertexArray,
    projection_matrix: Matrix4<f32>,
    /// Go back to world coordinates from normalized device coordinates.
    inverse_projection_matrix: Matrix4<f32>,
    // FIXME: use unsigned values for coordinates?
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
    fn end(&mut self);
    /// Converts screen coordinates to world coordinates.
    ///
    /// # Coordinates
    /// * Start form top left corner of the window.
    /// * Are in pixels.
    /// * Are at range [0, i32::MAX].
    // FIXME: use unsigned values for coordinates? SDL2 event makes i32 values.
    fn screen_coordinates_to_world_coordinates(&self, x: i32, y: i32) -> Point2<f32>;
    /// Enable or disable full screen mode.
    fn full_screen(&mut self, value: bool);
    /// Enable or disable vertical synchronization.
    fn v_sync(&mut self, value: bool);
    /// Screen width in world coordinates divided by 2.
    fn half_screen_width_world_coordinates(&self) -> f32;
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
            self.textures[Textures::Enemy as usize].bind();
            self.render_rectangle_with_texture(logic.get_enemy());

            if logic.get_enemy().get_laser_cannon_top().visible() {
                if logic.get_enemy().get_laser_cannon_top().green_color() {
                    self.textures[Textures::LaserCannonGreen as usize].bind();
                } else {
                    self.textures[Textures::LaserCannonRed as usize].bind();
                }
                self.render_rectangle_with_texture(logic.get_enemy().get_laser_cannon_top());
            }

            if logic.get_enemy().get_laser_cannon_bottom().visible() {
                if logic.get_enemy().get_laser_cannon_bottom().green_color() {
                    self.textures[Textures::LaserCannonGreen as usize].bind();
                } else {
                    self.textures[Textures::LaserCannonRed as usize].bind();
                }
                self.render_rectangle_with_texture(logic.get_enemy().get_laser_cannon_bottom());
            }

            if logic.get_enemy().get_shield().visible() {
                self.textures[Textures::Shield as usize].bind();
                self.render_rectangle_with_texture(logic.get_enemy().get_shield());
            }
        }

        self.color_shader.use_program();

        let color_blue = Vector3::new(0.0,0.0,1.0);
        let color_red = Vector3::new(1.0,0.0,0.0);
        let color_green = Vector3::new(0.0,0.5,0.0);
        let color_particle = Vector3::from_value(0.3);

        for laser in logic.get_player().get_lasers() {
            self.render_color_rectangle_with_color(laser, &color_green);
        }

        for laser in logic.get_enemy().get_lasers() {
            if let LaserColor::Red = laser.color() {
                self.render_color_rectangle_with_color(laser, &color_red);
            } else {
                self.render_color_rectangle_with_color(laser, &color_blue);
            }
        }

        for laser_bomb in logic.get_enemy().get_laser_bombs() {
            self.render_color_rectangle_with_color(laser_bomb, &color_blue);
        }

        if logic.get_explosion().visible() {
            for particle in logic.get_explosion().particles() {
                self.render_color_rectangle_with_color(particle, &color_particle);
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
                self.render_color_rectangle_with_color(*border, health_bar.color());
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
    fn end(&mut self) {
        self.window.gl_swap_window();

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
    /// Creates new OpenGLRenderer.
    ///
    /// This function will set OpenGL context version
    /// to OpenGL ES 2.0, if game is compiled with feature "gles".
    ///
    /// # Panics
    /// If window or OpenGL context creation fails.
    pub fn new(video_system: VideoSubsystem) -> OpenGLRenderer {
        let window = video_system.window("Space Boss Battles", DEFAULT_SCREEN_WIDTH as u32, DEFAULT_SCREEN_HEIGHT as u32).opengl().build().expect("window creation failed");

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

        let _context = window.gl_create_context().expect("opengl context creation failed");
        gl_raw::load_with(|name| video_system.gl_get_proc_address(name) as *const _);

        window.gl_make_current(&_context).expect("couldn't set opengl context to current thread");

        unsafe {
            gl_raw::ClearColor(0.0,0.0,0.0,1.0);
        }

        println!("OpenGL version: {:?}", gl::get_version_string());

        let mut renderer = OpenGLRenderer {
            video_system,
            window,
            _context,
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
        renderer.update_screen_size();
        renderer.update_projection_matrix();

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

    /// Updates fields `screen_width` and `screen_height` and
    /// OpenGL viewport to match current display mode.
    ///
    /// # Errors
    /// If getting the display mode fails this function will use
    /// `DEFAULT_SCREEN_WIDTH` and `DEFAULT_SCREEN_HEIGHT` const values.
    fn update_screen_size(&mut self) {
        let mut width = DEFAULT_SCREEN_WIDTH;
        let mut height = DEFAULT_SCREEN_HEIGHT;

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