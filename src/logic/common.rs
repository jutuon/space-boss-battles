/*
src/logic/common.rs, 2017-08-20

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Basic functionality for game logic.

use cgmath::prelude::*;
use cgmath::{Vector4, Matrix4, Rad, Vector2, BaseFloat, Point2, MetricSpace};

/// Should game object be destroyed?
pub trait CanDestroy {
    /// If this is true then game object should be destroyed.
    fn destroy(&self) -> bool;
}

/// Functions for accessing game object's position data.
pub trait GameObjectData<T: BaseFloat> {
    fn data(&self) -> &Data<T>;
    fn data_mut(&mut self) -> &mut Data<T>;
}

/// Basic game object functionality.
///
/// See also the `Data` struct documentation.
pub trait GameObject: GameObjectData<f32> {

    /// Moves game object to current direction.
    fn forward(&mut self, amount: f32) {
        self.data_mut().position += self.data().direction * amount;

        self.data_mut().update_model_matrix_position();
    }

    /// Turn game object's current direction. Angle is in radians.
    fn turn(&mut self, angle: f32) {
        self.data_mut().rotation += angle;

        self.data_mut().update_rotation(true);
    }

    /// Turns game object, but does not update model matrix, so
    /// game object won't look like it was turned.
    fn turn_without_updating_model_matrix(&mut self, angle: f32) {
        self.data_mut().rotation += angle;
        self.data_mut().update_rotation(false);
    }

    /// Return true if game object is outside the area defined by
    /// argument area.
    fn outside_allowed_area(&self, area: &Rectangle) -> bool {
        area.outside(&self.data().position)
    }

    /// If game object is outside the area defined by argument area then
    /// move it back to the nearest border of that area.
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

    /// Adds x and y values to game object's position.
    fn move_position(&mut self, x: f32, y: f32) {
        self.data_mut().position.x += x;
        self.data_mut().position.y += y;

        self.data_mut().update_model_matrix_position();
    }

    /// Sets x and y to be game object's current position.
    fn set_position(&mut self, x: f32, y: f32) {
        self.data_mut().position.x = x;
        self.data_mut().position.y = y;

        self.data_mut().update_model_matrix_position();
    }

    /// Sets game object position's y coordinate to be argument y.
    fn set_position_y(&mut self, y: f32) {
        self.data_mut().position.y = y;

        self.data_mut().update_model_matrix_position();
    }

    /// Check circle collision between game objects. Returns true if there is a collision.
    ///
    /// Game object's inner circle will be used to check the collision. See `Data` struct documentation
    /// for more details.
    fn circle_collision<T: GameObjectData<f32>>(&self, game_object: &T) -> bool {
        // Circle collision must use square root, so let's first check
        // collision with more efficient square collision.
        if !self.outer_axis_aligned_square_collision(game_object) {
            return false;
        }

        let distance = self.data().position.distance(game_object.data().position);

        distance <= self.data().radius_inner + game_object.data().radius_inner
    }

    /// Get position of current game object.
    fn position(&self) -> &Vector2<f32> {
        &self.data().position
    }

    /// Get game object position's x coordinate.
    fn x(&self) -> f32 {
        self.data().position.x
    }

    /// Get game object position's y coordinate.
    fn y(&self) -> f32 {
        self.data().position.y
    }

    /// Collision between two game object's outer axis aligned square. Returns true if there is a collision.
    ///
    /// Outer axis aligned square is square where game object's outer circle will fit. This square will not move when object is turned.
    /// For more details see `Data` struct documentation. This method is mainly used for making collision
    /// faster, by returning early if there is no collision.
    fn outer_axis_aligned_square_collision<T: GameObjectData<f32>>(&self, game_object: &T) -> bool {
        let x = self.data().position.x - game_object.data().position.x;
        let y = self.data().position.y - game_object.data().position.y;

        let objects_radius_sum = self.data().radius_outer + game_object.data().radius_outer;

        if x.abs() > objects_radius_sum || y.abs() > objects_radius_sum {
            return false;
        }

        true
    }
}

/// Axis aligned rectangle.
pub struct Rectangle {
    pub left_top_corner: Point2<f32>,
    pub right_bottom_corner: Point2<f32>,
}

impl Rectangle {
    /// Create new `Rectangle`.
    pub fn new (x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Rectangle {
        Rectangle {
            left_top_corner: Point2::new(x_min, y_max),
            right_bottom_corner: Point2::new(x_max, y_min),
        }
    }

    /// Return true if point is outside the rectangle.
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


/// Base data for all game objects, used for rendering, movement, turning and collision detection.
///
/// All game objects are rectangles that can be turned. `Data` will also contain some
/// precalculated properties that are useful for collision detection.
pub struct Data<T: BaseFloat> {
    /// Model matrix for rendering. Updating model matrix
    /// is required if there is visual position or rotation changes to game object.
    pub model_matrix: Matrix4<T>,
    pub position: Vector2<T>,
    /// Default direction is x unit vector.
    pub direction: Vector2<T>,
    pub width: T,
    pub height: T,
    /// Rotation is in radians.
    pub rotation: T,
    /// Radius of circle where game object rectangle will fit even if it would be turned.
    pub radius_outer: T,
    /// Radius of circle inside the game object rectangle.
    pub radius_inner: T,
}

impl Data<f32> {
    /// Create new `Data`.
    pub fn new(position: Vector2<f32>, width: f32, height: f32) -> Data<f32> {
        let x = width/2.0;
        let y = height/2.0;
        let radius_outer = f32::sqrt(x*x+y*y);

        let mut data = Data {
            model_matrix: Matrix4::identity(),
            position,
            direction: Vector2::unit_x(),
            width,
            height,
            rotation: 0.0,
            radius_outer,
            radius_inner: f32::min(width, height)/2.0,
        };

        data.update_rotation(true);

        data
    }

    /// Create new `Data` which width and height are the same.
    pub fn new_square(position: Vector2<f32>, side_length: f32) -> Data<f32> {
        Data::new(position, side_length, side_length)
    }

    /// Calculates new direction vector for game object.
    ///
    /// Model matrix will be updated if update_model_matrix argument is true.
    fn update_rotation(&mut self, update_model_matrix: bool) {
        let rotation_matrix = Matrix4::from_angle_z(Rad(self.rotation));

        self.direction = (rotation_matrix * Vector4::unit_x()).truncate().truncate();

        if update_model_matrix {
            let scale_matrix = Matrix4::from_nonuniform_scale(self.width, self.height, 1.0);
            self.model_matrix = rotation_matrix * scale_matrix;
            self.update_model_matrix_position();
        }
    }

    /// Updates model matrix position.
    fn update_model_matrix_position(&mut self) {
        self.model_matrix.w.x = self.position.x;
        self.model_matrix.w.y = self.position.y;
    }
}

/// Trait for nicer game object container updates.
pub trait UpdateContent<T> {
    /// Run check_object closure for every game object
    /// currently stored to the container implementing this trait.
    ///
    /// If closure returns true, the game object will be removed. This will be done with the index_buffer,
    /// where location indexes of game objects will be stored.
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