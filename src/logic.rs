/*
src/logic.rs, 2017-07-15

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use cgmath::prelude::*;
use cgmath::{Vector4, Matrix4};

use input::Input;

pub struct Logic {
    player: Player,
}

impl Logic {
    pub fn new() -> Logic {
        let player = Player::new();
        Logic { player }
    }

    pub fn update<T: Input>(&mut self, input: &T) {
        self.player.update(input);
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }
}

pub struct Player {
    model_matrix: Matrix4<f32>,
    speed: f32,
}

impl Player {
    fn new() -> Player {
        let model_matrix = Matrix4::identity();
        let speed = 0.1;
        Player { model_matrix, speed }
    }

    fn update(&mut self, input: &Input) {
        let speed = self.speed;

        if input.up() {
            self.position_mut().y += speed;
        } else if input.down() {
            self.position_mut().y -= speed;
        }

        if input.left() {
            self.position_mut().x -= speed;
        } else if input.right(){
            self.position_mut().x += speed;
        }

    }
}


pub trait GameObject<T> {
    fn model_matrix(&self) -> &Matrix4<T>;
    fn position_mut(&mut self) -> &mut Vector4<T>;
    fn position(&self) -> &Vector4<T>;
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
}


impl GameObject<f32> for Player {
    fn model_matrix(&self) -> &Matrix4<f32> {
        &self.model_matrix
    }

    fn position_mut(&mut self) -> &mut Vector4<f32> {
        &mut self.model_matrix.w
    }

    fn position(&self) -> &Vector4<f32> {
        &self.model_matrix.w
    }

    fn x(&self) -> f32 {
        self.model_matrix.w.x
    }

    fn y(&self) -> f32 {
        self.model_matrix.w.y
    }

    fn z(&self) -> f32 {
        self.model_matrix.w.z
    }
}
