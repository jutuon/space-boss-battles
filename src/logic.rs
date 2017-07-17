/*
src/logic.rs, 2017-07-17

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use cgmath::prelude::*;
use cgmath::{Vector4, Matrix4};

use input::Input;

use time::PreciseTime;

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
    lasers: Vec<Laser>,
    laser_timer: PreciseTime,
}

impl Player {
    fn new() -> Player {
        let model_matrix = Matrix4::identity();
        let speed = 0.1;
        let lasers = vec![];
        let laser_timer = PreciseTime::now();
        Player { model_matrix, speed, lasers, laser_timer }
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
        let current_time = PreciseTime::now();

        if input.shoot() && self.laser_timer.to(current_time).num_milliseconds() >= 500 {
            self.lasers.push(Laser::new(self.model_matrix.w.x + 1.0, self.model_matrix.w.y));
            self.laser_timer = current_time;
        }

        self.check_position();

        self.clean_and_update_lasers(input);
    }

    fn check_position(&mut self) {
        let &Vector4{x, y, ..} = self.position();

        let width = 10.0;

        if x > width {
            self.position_mut().x = width;
        } else if x < -width {
            self.position_mut().x = -width;
        }

        let height = 8.0;

        if y > height {
            self.position_mut().y = height;
        } else if y < -height {
            self.position_mut().y = -height;
        }
    }

    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self, input: &Input) {

        let mut remove = (false, 0);

        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.update(input);

            if laser.destroy() {
                remove = (true, i);
            }
        }

        if let (true, i) = remove {
            self.lasers.swap_remove(i);
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


pub struct Laser {
    model_matrix: Matrix4<f32>,
    speed: f32,
    destroy: bool,
}

impl Laser {
    fn new(x: f32, y: f32) -> Laser {
        let mut model_matrix = Matrix4::identity();
        model_matrix.w.x = x;
        model_matrix.w.y = y;

        model_matrix.x.x = 0.3;
        model_matrix.y.y = 0.1;

        let speed = 0.1;
        let destroy = false;
        Laser { model_matrix, speed, destroy }
    }

    fn update(&mut self, input: &Input) {
        let speed = self.speed;

        self.position_mut().x += speed;

        self.check_position();
    }

    fn check_position(&mut self) {
        let &Vector4{x: x, y: y, ..} = self.position();

        let width = 10.0;

        if  x > width || x < -width {
            self.destroy = true;
        }

        let height = 8.0;

        if  y > height || y < -height {
            self.destroy = true;
        }
    }

    pub fn destroy(&self) -> bool {
        self.destroy
    }
}

impl GameObject<f32> for Laser {
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