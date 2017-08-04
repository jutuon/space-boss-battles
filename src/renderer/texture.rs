/*
src/renderer/texture.rs, 2017-08-01

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use std::fs::File;

use gl::texture::*;

use image::png::PNGDecoder;
use image::{ImageDecoder, DecodingResult, ColorType};


pub enum Textures {
    Player,
    Enemy,
    Background,
    Font,
    TextureCount,
}

impl Textures {
    pub fn load_all() -> [Texture; Textures::TextureCount as usize] {
        [
            Textures::load("game_files/images/player.png"),
            Textures::load("game_files/images/enemy1.png"),
            Textures::load("game_files/images/background.png"),
            Textures::load("game_files/images/tilemap-font.png"),
        ]
    }

    fn load(file_path: &str) -> Texture {
        let img_file = File::open(file_path).expect("img opening fail");
        let mut img = PNGDecoder::new(img_file);

        let (width, height) = img.dimensions().expect("img dimensions fail");

        let mut rgba;
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