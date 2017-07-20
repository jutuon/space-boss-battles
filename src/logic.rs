/*
src/logic.rs, 2017-07-20

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use cgmath::prelude::*;
use cgmath::{Vector4, Matrix4, Rad, Vector2, BaseFloat, Point2};

use input::Input;

use time::PreciseTime;
use Timer;

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
    data: Data<f32>,
    speed: f32,
    lasers: Vec<Laser>,
    laser_timer: Timer,
}

impl Player {
    fn new() -> Player {
        let data = Data::new(0.0,0.0,1.0,1.0);
        let speed = 0.05;
        let lasers = vec![];
        let laser_timer = Timer::new();
        Player { data, speed, lasers, laser_timer }
    }

    fn update(&mut self, input: &Input) {
        let speed = self.speed;

        let mut y_speed = 0.0;
        if input.up() {
            y_speed = speed;
        } else if input.down() {
            y_speed = -speed;
        }

        let mut x_speed = 0.0;
        if input.left() {
            x_speed = -speed;
        } else if input.right(){
            x_speed = speed;
        }

        self.move_position(x_speed, y_speed);

        if input.shoot() && self.laser_timer.check(PreciseTime::now(), 400) {
            let laser = Laser::new(self.data().position.x + 1.0, self.data().position.y);
            self.lasers.push(laser);
        }

        let width = 5.0;
        let height = 4.0;
        let area = Rectangle::new(-width, width, -height, height);
        self.stay_at_area(&area);

        self.clean_and_update_lasers();
    }

    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self) {
        let mut remove = (false, 0);

        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.update();

            if laser.destroy() {
                remove = (true, i);
            }
        }

        if let (true, i) = remove {
            self.lasers.swap_remove(i);
        }
    }
}





impl GameObject for Player {}
impl ModelMatrix for Player {}


impl GameObjectData<f32> for Player {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}

pub struct Laser {
    data: Data<f32>,
    speed: f32,
    destroy: bool,
}

impl Laser {
    fn new(x: f32, y: f32) -> Laser {
        let data = Data::new(x, y, 0.3, 0.1);
        let speed = 0.08;
        let destroy = false;
        Laser { data, speed, destroy }
    }

    fn update(&mut self) {
        let speed = self.speed;
        self.forward(speed);

        let width = 5.0;
        let height = 4.0;
        let area = Rectangle::new(-width, width, -height, height);

        if self.outside_allowed_area(&area) {
            self.destroy = true;
        }
    }
}

impl CanDestroy for Laser {
    fn destroy(&self) -> bool {
        self.destroy
    }
}

impl GameObjectData<f32> for Laser {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}

impl GameObject for Laser {}
impl ModelMatrix for Laser {}





pub trait ModelMatrix
    where Self: GameObjectData<f32> {
    fn model_matrix(&self) -> &Matrix4<f32> {
        &self.data().model_matrix
    }
}

pub trait CanDestroy {
    fn destroy(&self) -> bool;
}

pub trait GameObjectData<T: BaseFloat> {
    fn data(&self) -> &Data<T>;
    fn data_mut(&mut self) -> &mut Data<T>;
}

pub trait GameObject
    where Self: GameObjectData<f32> {

    fn forward(&mut self, amount: f32) {
        self.data_mut().position += self.data().direction * amount;

        self.data_mut().update_model_matrix_position();
    }

    fn turn(&mut self, angle: f32) {
        self.data_mut().rotation += angle;

        self.data_mut().update_rotation();
    }

    fn outside_allowed_area(&self, area: &Rectangle) -> bool {
        area.outside(&self.data().position)
    }

    fn stay_at_area(&mut self, area: &Rectangle) {
        let x = self.data().position.x;

        let mut position_changed = false;

        if x < area.left_top_corner.x {
            self.data_mut().position.x = area.left_top_corner.x;
            position_changed = true;
        } else if area.right_bottom_corner.x < x {
            self.data_mut().position.x = area.right_bottom_corner.x;
            position_changed = true;
        }

        let y = self.data().position.y;

        if y < area.right_bottom_corner.y {
            self.data_mut().position.y = area.right_bottom_corner.y;
            position_changed = true;
        } else if area.left_top_corner.y < y {
            self.data_mut().position.y = area.left_top_corner.y;
            position_changed = true;
        }

        if position_changed {
            self.data_mut().update_model_matrix_position();
        }
    }

    fn move_position(&mut self, x: f32, y: f32) {
        self.data_mut().position.x += x;
        self.data_mut().position.y += y;

        self.data_mut().update_model_matrix_position();
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.data_mut().position.x = x;
        self.data_mut().position.y = y;

        self.data_mut().update_model_matrix_position();
    }
}



pub struct Rectangle {
    pub left_top_corner: Point2<f32>,
    pub right_bottom_corner: Point2<f32>,
}

impl Rectangle {
    fn new (x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Rectangle {
        Rectangle {
            left_top_corner: Point2::new(x_min, y_max),
            right_bottom_corner: Point2::new(x_max, y_min),
        }
    }

    fn outside(&self, point: &Point2<f32>) -> bool {
        if point.x < self.left_top_corner.x || self.right_bottom_corner.x < point.x {
            return true;
        }

        if point.y < self.right_bottom_corner.y || self.left_top_corner.y < point.y {
            return true;
        }

        false
    }
}

pub struct Data<T: BaseFloat> {
    pub model_matrix: Matrix4<T>,
    pub position: Point2<T>,
    pub direction: Vector2<T>,
    pub width: T,
    pub height: T,
    pub rotation: T,
}

impl Data<f32> {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Data<f32> {
        let model_matrix = Matrix4::identity();
        let position = Point2::new(x, y);
        let direction = Vector2::unit_x();
        let rotation = 0.0;

        let mut data = Data {model_matrix, position, direction, width, height, rotation};
        data.update_rotation();

        data
    }

    fn update_rotation(&mut self) {
        let rotation_matrix = Matrix4::from_angle_z(Rad(self.rotation));

        self.direction = (rotation_matrix * Vector4::unit_x()).truncate().truncate();

        let scale_matrix = Matrix4::from_nonuniform_scale(self.width, self.height, 1.0);
        self.model_matrix = rotation_matrix * scale_matrix;

        self.update_model_matrix_position();
    }

    fn update_model_matrix_position(&mut self) {
        self.model_matrix.w.x = self.position.x;
        self.model_matrix.w.y = self.position.y;
    }
}