/*
src/gui/components.rs, 2017-08-05

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
    text: GUIText,
}

impl GUIButton {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: &str) -> GUIButton {
        let mut button = GUIButton {
            rectangle: GUIRectangle::new(x, y, width, height),
            state: GUIComponentState::Normal,
            text: GUIText::new(x, y, text),
        };

        button.set_state(GUIComponentState::Normal);

        button
    }

    pub fn model_matrix(&self) -> &Matrix4<f32> {
        &self.rectangle.model_matrix()
    }

    pub fn color(&self) -> &Vector3<f32> {
        &self.rectangle.color()
    }

    pub fn get_text(&self) -> &GUIText {
        &self.text
    }
}

pub trait GUICollision {
    fn collision(&self, point: &Point2<f32>) -> bool;
}

impl GUICollision for GUIButton {
    fn collision(&self, point: &Point2<f32>) -> bool {
        self.rectangle.axis_aligned_rectangle_and_point_collision(point)
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

pub struct GUIGroupBuilder<T: SetGUIComponentState> {
    components: Vec<T>,
}

impl <T: SetGUIComponentState> GUIGroupBuilder<T> {
    pub fn new() -> GUIGroupBuilder<T> {
        let mut vec = Vec::new();

        GUIGroupBuilder {
            components: vec,
        }
    }

    pub fn add(&mut self, gui_component: T) {
        self.components.push(gui_component);

    }

    pub fn create_gui_group(mut self) -> GUIGroup<T> {
        if self.components.len() == 0 {
            panic!("GUIGroup can't be empty.");
        }

        self.components[0].set_state(GUIComponentState::Selected);

        GUIGroup {
            components: self.components,
            selected: 0,
        }
    }
}

pub struct GUIGroup<T: SetGUIComponentState> {
    components: Vec<T>,
    selected: usize,
}

impl <T: SetGUIComponentState + GUICollision> GUIGroup<T> {
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

    pub fn get_collision_index(&self, point: &Point2<f32>) -> Option<usize> {
        for (i, button) in self.components.iter().enumerate() {
            if button.collision(point) {
                return Some(i);
            }
        };

        None
    }

    pub fn update_selection(&mut self, point: &Point2<f32>) {
        let mut index = None;
        for (i, button) in self.components.iter().enumerate() {
            if button.collision(point) {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            self.components[self.selected].set_state(GUIComponentState::Normal);
            self.selected = i;
            self.components[self.selected].set_state(GUIComponentState::Selected);
        }
    }
}


pub struct Tile {
    rectangle: GUIRectangle<f32>,
    tile_info: Vector3<f32>,
}

impl Tile {
    pub fn new(index: (u32, u32), gui_rectangle: GUIRectangle<f32>) -> Tile {
        let tile_size = 1.0/16.0;
        let x_movement = tile_size * index.0 as f32;
        let y_movement = 1.0 - tile_size * (index.1 + 1) as f32;

        Tile {
            rectangle: gui_rectangle,
            tile_info: Vector3::new(x_movement, y_movement, tile_size),
        }
    }

    pub fn get_rectangle(&self) -> &GUIRectangle<f32> {
        &self.rectangle
    }

    pub fn get_tile_info(&self) -> &Vector3<f32> {
        &self.tile_info
    }
}


fn tilemap_index_from_char(c: char) -> (u32, u32) {
    match c {
        '0' => (0,0),
        '1' => (1,0),
        '2' => (2,0),
        '3' => (3,0),
        '4' => (4,0),
        '5' => (5,0),
        '6' => (6,0),
        '7' => (7,0),
        '8' => (8,0),
        '9' => (9,0),
        'A' => (10,0),
        'B' => (11,0),
        'C' => (12,0),
        'D' => (13,0),
        'E' => (14,0),
        'F' => (15,0),

        'G' => (0,1),
        'H' => (1,1),
        'I' => (2,1),
        'J' => (3,1),
        'K' => (4,1),
        'L' => (5,1),
        'M' => (6, 1),
        'N' => (7, 1),
        'O' => (8, 1),
        'P' => (9, 1),
        'Q' => (10,1),
        'R' => (11,1),
        'S' => (12,1),
        'T' => (13,1),
        'U' => (14,1),
        'V' => (15,1),

        'W' => (0,2),
        'X' => (1,2),
        'Y' => (2,2),
        'Z' => (3,2),
        ' ' => (4,2),
        'a' => (5, 2),
        'b' => (6, 2),
        'c' => (7, 2),
        'd' => (8, 2),
        'e' => (9,2),
        'f' => (10,2),
        'g' => (11,2),
        'h' => (12,2),
        'i' => (13,2),
        'j' => (14,2),
        'k' => (15, 2),

        'l' => (0, 3),
        'm' => (1, 3),
        'n' => (2, 3),
        'o' => (3, 3),
        'p' => (4, 3),
        'q' => (5, 3),
        'r' => (6, 3),
        's' => (7, 3),
        't' => (8, 3),
        'u' => (9, 3),
        'v' => (10,3),
        'w' => (11,3),
        'x' => (12,3),
        'y' => (13,3),
        'z' => (14,3),

        _ => tilemap_index_from_char(' '),

    }
}


pub struct GUIText {
    tiles: Vec<Tile>,
    x: f32,
    y: f32,
    size: f32,
    width: f32,
}

impl GUIText {
    pub fn new(x: f32, y: f32, text: &str) -> GUIText {
        let mut gui_text = GUIText {
            tiles: Vec::new(),
            x: x,
            y: y,
            size: 0.57,
            width: 0.0,
        };

        gui_text.change_text(text);

        gui_text
    }

    pub fn change_text(&mut self, text: &str) {
        self.tiles.clear();

        let text_len = text.len() as f32;

        let size = self.size - 0.17;
        self.width = text_len * size;
        let mut x = self.x - self.width/2.0;

        for c in text.chars() {
            let rectangle = GUIRectangle::new(x, self.y, self.size, self.size);

            self.tiles.push(Tile::new(tilemap_index_from_char(c), rectangle));

            x += size;
        }
    }

    pub fn get_tiles(&self) -> &Vec<Tile> {
        &self.tiles
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }
}

pub struct GUIFpsCounter {
    fps_text: GUIText,
    fps_count_text: GUIText,
    show_fps: bool,
}

impl GUIFpsCounter {
    pub fn new(x: f32, y:f32) -> GUIFpsCounter {
        let fps_text = GUIText::new(x, y, "FPS ");
        let fps_count_text = GUIText::new(x + fps_text.get_width(), y, "0");

        let show_fps = false;

        GUIFpsCounter {
            fps_text,
            fps_count_text,
            show_fps,
        }
    }

    pub fn update_fps_count(&mut self, fps_count: u32) {
        let text = fps_count.to_string();
        self.fps_count_text.change_text(&text);
    }

    pub fn texts(&self) -> [&GUIText; 2] {
        [&self.fps_text, &self.fps_count_text]
    }

    pub fn show_fps(&self) -> bool {
        self.show_fps
    }

    pub fn set_show_fps(&mut self, value: bool) {
        self.show_fps = value;
    }
}