/*
src/logic/common.rs, 2017-08-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use cgmath::prelude::*;
use cgmath::{Vector4, Matrix4, Rad, Vector2, BaseFloat, Point2, MetricSpace};

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

        self.data_mut().update_rotation(true);
    }

    fn turn_without_updating_model_matrix(&mut self, angle: f32) {
        self.data_mut().rotation += angle;
        self.data_mut().update_rotation(false);
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

    fn set_position_y(&mut self, y: f32) {
        self.data_mut().position.y = y;

        self.data_mut().update_model_matrix_position();
    }

    fn circle_collision<T: GameObjectData<f32>>(&self, game_object: &T) -> bool {
        if !self.outer_square_collision(game_object) {
            return false;
        }

        let distance = self.data().position.distance(game_object.data().position);

        distance <= self.data().radius_inner + game_object.data().radius_inner
    }

    fn position(&self) -> &Vector2<f32> {
        &self.data().position
    }

    fn x(&self) -> f32 {
        self.data().position.x
    }

    fn y(&self) -> f32 {
        self.data().position.y
    }

/*
    fn circle_point(&self, point: Point2<f32>) -> bool {
        let a = self.data().position.x;
        let b = self.data().position.y;

        let r = self.data().width;

        let x_min_a = point.x - a;
        let y_min_b = point.y - b;

        x_min_a * x_min_a + y_min_b * y_min_b <= r*r
    }

    fn axis_aligned_rectangle_collision<T: GameObjectData<f32>>(&self, game_object: &T) -> bool {
        let x = self.data().position.x - game_object.data().position.x;
        let y = self.data().position.y - game_object.data().position.y;

        let objects_width_half = self.data().width/2.0 + game_object.data().width/2.0;
        let objects_height_half = self.data().height/2.0 + game_object.data().height/2.0;

        if x.abs() > objects_width_half || y.abs() > objects_height_half {
            return false;
        }

        true
    }
*/
    fn outer_square_collision<T: GameObjectData<f32>>(&self, game_object: &T) -> bool {
        let x = self.data().position.x - game_object.data().position.x;
        let y = self.data().position.y - game_object.data().position.y;

        let objects_radius_sum = self.data().radius_outer + game_object.data().radius_outer;

        if x.abs() > objects_radius_sum || y.abs() > objects_radius_sum {
            return false;
        }

        true
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

    fn outside(&self, point: &Vector2<f32>) -> bool {
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
    pub position: Vector2<T>,
    pub direction: Vector2<T>,
    pub width: T,
    pub height: T,
    pub rotation: T,
    pub radius_outer: T,
    pub radius_inner: T,
}

impl Data<f32> {
    pub fn new(position: Vector2<f32>, width: f32, height: f32) -> Data<f32> {
        let model_matrix = Matrix4::identity();
        let direction = Vector2::unit_x();
        let rotation = 0.0;

        let x = width/2.0;
        let y = height/2.0;
        let radius_outer = f32::sqrt(x*x+y*y);

        let radius_inner = f32::min(width, height)/2.0;

        let mut data = Data {model_matrix, position, direction, width, height, rotation, radius_outer, radius_inner};
        data.update_rotation(true);

        data
    }

    pub fn new_square(position: Vector2<f32>, side_length: f32) -> Data<f32> {
        Data::new(position, side_length, side_length)
    }

    fn update_rotation(&mut self, update_model_matrix: bool) {
        let rotation_matrix = Matrix4::from_angle_z(Rad(self.rotation));

        self.direction = (rotation_matrix * Vector4::unit_x()).truncate().truncate();

        if update_model_matrix {
            let scale_matrix = Matrix4::from_nonuniform_scale(self.width, self.height, 1.0);
            self.model_matrix = rotation_matrix * scale_matrix;
        }

        self.update_model_matrix_position();
    }

    fn update_model_matrix_position(&mut self) {
        self.model_matrix.w.x = self.position.x;
        self.model_matrix.w.y = self.position.y;
    }
}

pub trait UpdateContent<T> {
    fn update(&mut self, index_buffer: &mut Vec<usize>, check_object: &mut FnMut(&mut T) -> bool);
}

impl <T> UpdateContent<T> for Vec<T> {
    fn update(&mut self, index_buffer: &mut Vec<usize>, check_object: &mut FnMut(&mut T) -> bool) {
        for (i, object) in self.iter_mut().enumerate() {
            if check_object(object) {
                index_buffer.push(i);
            }
        }

        for i in index_buffer.iter().rev() {
            self.swap_remove(*i);
        }

        index_buffer.clear();
    }
}