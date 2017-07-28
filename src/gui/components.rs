/*
src/gui/components.rs, 2017-07-28

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use cgmath::{Matrix4, Point2, Vector3};
use cgmath::prelude::*;


pub enum GUIComponentState {
    MouseOver,
    Selected,
    Normal,
}

pub struct GUIRectangle<T> {
    model_matrix: Matrix4<T>,
    position: Point2<T>,
    width: T,
    height: T,
    color: Vector3<T>,
}

impl GUIRectangle<f32> {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> GUIRectangle<f32> {
        let model_matrix = Matrix4::identity();
        let position = Point2::new(x, y);

        let color = Vector3::zero();

        let mut rectangle = GUIRectangle { model_matrix, position, width, height, color };
        rectangle.update_model_matrix();

        rectangle
    }

    fn update_model_matrix(&mut self) {
        self.model_matrix = Matrix4::from_nonuniform_scale(self.width, self.height, 1.0);

        self.model_matrix.w.x = self.position.x;
        self.model_matrix.w.y = self.position.y;
    }

    fn axis_aligned_rectangle_and_point_collision(&self, point: &Point2<f32>) -> bool {
        let x = self.position.x - point.x;
        let y = self.position.y - point.y;

        let objects_width_half = self.width/2.0;
        let objects_height_half = self.height/2.0;

        if x.abs() > objects_width_half || y.abs() > objects_height_half {
            return false;
        }

        true
    }

    pub fn model_matrix(&self) -> &Matrix4<f32> {
        &self.model_matrix
    }

    pub fn color(&self) -> &Vector3<f32> {
        &self.color
    }

    pub fn set_color(&mut self, color: Vector3<f32>) {
        self.color = color;
    }


}

pub struct GUIButton {
    rectangle: GUIRectangle<f32>,
    state: GUIComponentState,
}

impl GUIButton {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> GUIButton {
        let mut button = GUIButton {
            rectangle: GUIRectangle::new(x, y, width, height),
            state: GUIComponentState::Normal
        };

        button.set_state(GUIComponentState::Normal);

        button
    }

    fn collision(&self, point: &Point2<f32>) -> bool {
        self.rectangle.axis_aligned_rectangle_and_point_collision(point)
    }

    pub fn model_matrix(&self) -> &Matrix4<f32> {
        &self.rectangle.model_matrix()
    }

    pub fn color(&self) -> &Vector3<f32> {
        &self.rectangle.color()
    }

}

pub trait SetGUIComponentState {
    fn set_state(&mut self, state: GUIComponentState);
}

impl SetGUIComponentState for GUIButton {
    fn set_state(&mut self, state: GUIComponentState) {
        let color_mouse_over = Vector3::new(0.0,0.5,0.0);
        let color_selected = Vector3::new(0.0,0.0,1.0);
        let color_normal = Vector3::new(0.0,0.0,0.4);

        match state {
            GUIComponentState::Normal => self.rectangle.set_color(color_normal),
            GUIComponentState::Selected => self.rectangle.set_color(color_selected),
            GUIComponentState::MouseOver => self.rectangle.set_color(color_mouse_over),
        }

    }
}


pub struct GUIGroup<T: SetGUIComponentState> {
    components: Vec<T>,
    selected: usize,
}

impl <T: SetGUIComponentState> GUIGroup<T> {
    pub fn new(mut first_component: T) -> GUIGroup<T> {
        let mut vec = Vec::new();
        first_component.set_state(GUIComponentState::Selected);
        vec.push(first_component);


        GUIGroup {
            components: vec,
            selected: 0,
        }
    }

    pub fn add(mut self, component: T) -> GUIGroup<T> {
        self.components.push(component);
        self
    }

    pub fn selection_up(&mut self) {
        self.update_selection_index(true);
    }

    pub fn selection_down(&mut self) {
        self.update_selection_index(false);
    }

    fn update_selection_index(&mut self, selection_up: bool) {
        self.components[self.selected].set_state(GUIComponentState::Normal);

        self.selected = if selection_up {
            if self.selected == 0 {
                self.components.len() - 1
            } else {
                self.selected - 1
            }
        } else {
            let i = self.selected + 1;
            if i >= self.components.len() {
                0
            } else {
                i
            }
        };

        self.components[self.selected].set_state(GUIComponentState::Selected);
    }

    pub fn get_selection_index(&self) -> usize {
        self.selected
    }

    pub fn get_components(&self) -> &[T] {
        self.components.as_slice()
    }
}


