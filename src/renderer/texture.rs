/*
src/renderer/texture.rs, 2017-08-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Textures for `OpenGLRenderer`.

use std::fs::File;

use gl::texture::*;

use image::png::PNGDecoder;
use image::{ImageDecoder, DecodingResult, ColorType};

/// Available textures.
pub enum Textures {
    Player,
    Enemy,
    EnemyWithShield,
    Background,
    Font,
    Shield,
    LaserCannonGreen,
    LaserCannonRed,
    LaserBomb,
    TextureCount,
}

impl Textures {
    /// Loads textures to an array.
    ///
    /// # Panics
    /// * If loading of some texture fails.
    pub fn load_all() -> [Texture; Textures::TextureCount as usize] {
        [
            Textures::load("game_files/images/player.png"),
            Textures::load("game_files/images/enemy1.png"),
            Textures::load("game_files/images/enemy2.png"),
            Textures::load("game_files/images/background.png"),
            Textures::load("game_files/images/tilemap-font.png"),
            Textures::load("game_files/images/shield.png"),
            Textures::load("game_files/images/laser_cannon_green.png"),
            Textures::load("game_files/images/laser_cannon_red.png"),
            Textures::load("game_files/images/laser_bomb.png"),
        ]
    }

    /// Load texture from from RGBA or RGB image with PNG format.
    ///
    /// # Panics
    /// * Opening the file fails.
    /// * Can't read image dimensions, color type or data.
    /// * If image data is not unsigned bytes.
    /// * Image color type is not RGBA or RGB.
    fn load(file_path: &str) -> Texture {
        let img_file = File::open(file_path).expect("img opening fail");
        let mut img = PNGDecoder::new(img_file);

        let (width, height) = img.dimensions().expect("img dimensions fail");

        let rgba;
        match img.colortype().expect("img color type fail") {
            ColorType::RGBA(_) => rgba = true,
            ColorType::RGB(_) => rgba = false,
            _ => panic!("image's color type is not RGB or RGBA"),
        }

        let img_data_result = img.read_image().expect("img decoding fail");

        let img_data = match img_data_result {
            DecodingResult::U8(data) => data,
            _ => panic!("unknown image data"),
        };

        Texture::new(width, height, img_data, rgba)
    }
}