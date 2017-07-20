/*
src/renderer/texture.rs, 2017-07-20

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
    TextureCount,
}

impl Textures {
    pub fn load_all() -> [TextureRGBA; Textures::TextureCount as usize] {
        [Textures::load("game_files/images/player.png"),
        Textures::load("game_files/images/enemy1.png")]
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