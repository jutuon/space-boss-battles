/*
src/logic/common.rs, 2017-07-20

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use cgmath::prelude::*;
use cgmath::{Vector4, Matrix4, Rad, Vector2, BaseFloat, Point2};

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

    fn stay_at_area(&mut self, area: &Rectangle) -> bool {
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

        position_changed
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
    pub fn new (x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Rectangle {
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
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Data<f32> {
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