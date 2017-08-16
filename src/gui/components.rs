/*
src/gui/components.rs, 2017-08-16

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! GUI toolkit components.

use cgmath::{Matrix4, Point2, Vector3};
use cgmath::prelude::*;

use renderer::{ModelMatrix, Color, TileLocationInfo};

use super::GUIEvent;

/// Macro for implementing `ModelMatrix` trait.
macro_rules! impl_model_matrix {
    ( $x:ty ) => {
        impl ModelMatrix for $x {
            fn model_matrix(&self) -> &Matrix4<f32> {
                &self.model_matrix
            }
        }
    };
    ( $x:ty, $location:ident ) => {
        impl ModelMatrix for $x {
            fn model_matrix(&self) -> &Matrix4<f32> {
                &self.$location.model_matrix()
            }
        }
    };
}

/// Macro for implementing `Color` trait.
macro_rules! impl_color {
    ( $x:ty ) => {
        impl Color for $x {
            fn color(&self) -> &Vector3<f32> {
                &self.color
            }
        }
    }
}

pub trait GUIActionData<T: Clone + Copy> {
    fn action_data(&self) -> T;
    fn set_action_data(&mut self, data: T);
}

pub trait GUIComponent {
    fn collision(&self, point: &Point2<f32>) -> bool;
    fn set_state(&mut self, state: GUIComponentState);
}

pub trait GUIPosition {
    fn update_position_from_half_screen_width(&mut self, width: f32);
    fn width(&self) -> f32;
    fn set_x(&mut self, x: f32);
    fn alignment(&self) -> GUIComponentAlignment;

    fn calculate_component_position(&self, new_x: f32, width: f32) -> f32 {
        let mut x = new_x;

        let half_width = width/2.0;

        match self.alignment() {
            GUIComponentAlignment::Left => {
                x += half_width;
            },
            GUIComponentAlignment::Right => {
                x -= half_width;
            },
            _  => (),
        };

        x
    }

    fn update_component_position(&mut self, new_x: f32) {
        let x = self.calculate_component_position(new_x, self.width());
        self.set_x(x);
    }
}


pub enum GUIComponentState {
    Selected,
    Normal,
}

#[derive(Copy, Clone)]
pub enum GUIComponentAlignment {
    Left,
    Right,
    Center,
}

pub struct GUIRectangle<T> {
    model_matrix: Matrix4<T>,
    position: Point2<T>,
    width: T,
    height: T,
}

impl GUIRectangle<f32> {
    pub fn new(position: Point2<f32>, width: f32, height: f32) -> GUIRectangle<f32> {
        let mut rectangle = GUIRectangle {
            model_matrix: Matrix4::identity(),
            position,
            width,
            height
        };
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

    fn position_mut(&mut self) -> &mut Point2<f32> {
        &mut self.position
    }

    fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    fn width(&self) -> f32 {
        self.width
    }
}

impl_model_matrix!(GUIRectangle<f32>);


pub struct GUIButton<T: Clone + Copy> {
    rectangle: GUIRectangle<f32>,
    text: GUIText,
    color: Vector3<f32>,
    action_data: T,
}

impl <T: Clone + Copy> GUIButton<T> {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: &str, action_data: T) -> GUIButton<T> {
        let mut button = GUIButton {
            rectangle: GUIRectangle::new(Point2 {x, y}, width, height),
            text: GUIText::new(x, y, text),
            color: Vector3::zero(),
            action_data,
        };

        button.set_state(GUIComponentState::Normal);

        button
    }

    pub fn get_text(&self) -> &GUIText {
        &self.text
    }
}

impl_model_matrix!(GUIButton<GUIEvent>, rectangle);
impl_color!(GUIButton<GUIEvent>);

impl <T: Clone + Copy> GUIActionData<T> for GUIButton<T> {
    fn action_data(&self) -> T {
        self.action_data
    }

    fn set_action_data(&mut self, data: T) {
        self.action_data = data;
    }
}

impl <T: Clone + Copy> GUIComponent for GUIButton<T> {
    fn collision(&self, point: &Point2<f32>) -> bool {
        self.rectangle.axis_aligned_rectangle_and_point_collision(point)
    }

    fn set_state(&mut self, state: GUIComponentState) {
        let color_selected = Vector3::new(0.0,0.0,1.0);
        let color_normal = Vector3::new(0.0,0.0,0.4);

        match state {
            GUIComponentState::Normal => self.color = color_normal,
            GUIComponentState::Selected => self.color = color_selected,
        }
    }
}


pub struct GUIGroupBuilder<T: GUIComponent> {
    components: Vec<T>,
}

impl <T: GUIComponent> GUIGroupBuilder<T> {
    pub fn new() -> GUIGroupBuilder<T> {
        GUIGroupBuilder {
            components: Vec::new(),
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

pub struct GUIGroup<T: GUIComponent> {
    components: Vec<T>,
    selected: usize,
}

impl <T: GUIComponent> GUIGroup<T> {
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

    pub fn get_components(&self) -> &[T] {
        self.components.as_slice()
    }

    pub fn get_components_mut(&mut self) -> &mut [T] {
        self.components.as_mut_slice()
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

impl <T: GUIComponent + GUIActionData<GUIEvent>> GUIGroup<T> {
    pub fn set_action_of_currently_selected_component(&mut self, action: GUIEvent) {
        self.components[self.selected].set_action_data(action);
    }

    pub fn action_of_currently_selected_component(&self) -> GUIEvent {
        self.components[self.selected].action_data()
    }

    pub fn check_collision_and_return_action(&self, point: &Point2<f32>) -> Option<GUIEvent> {
        for button in &self.components {
            if button.collision(point) {
                return Some(button.action_data());
            }
        };

        None
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

    pub fn set_gui_rectangle(&mut self, gui_rectangle: GUIRectangle<f32>) {
        self.rectangle = gui_rectangle;
    }
}

impl_model_matrix!(Tile, rectangle);

impl TileLocationInfo for Tile {
    fn tile_info(&self) -> &Vector3<f32> {
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
    position: Point2<f32>,
    font_size: f32,
    space_between_tiles: f32,
    width: f32,
    alignment: GUIComponentAlignment,
}

impl GUIText {
    pub fn new(x: f32, y: f32, text: &str) -> GUIText {
        GUIText::new_with_alignment(x, y, text, GUIComponentAlignment::Center)
    }

    pub fn new_with_alignment(x: f32, y: f32, text: &str, alignment: GUIComponentAlignment) -> GUIText {
        let mut gui_text = GUIText {
            tiles: Vec::new(),
            position: Point2 {x, y: y - 0.04},
            font_size: 0.57,
            space_between_tiles: 0.0,
            width: 0.0,
            alignment,
        };

        gui_text.change_text(text);

        gui_text
    }

    pub fn change_text(&mut self, text: &str) {
        self.tiles.clear();

        let text_len = text.len() as f32;

        self.space_between_tiles = self.font_size - 0.17;
        self.width = text_len * self.space_between_tiles;

        let mut x = self.calculate_component_position(self.position.x, self.width);

        for c in text.chars() {
            let rectangle = GUIRectangle::new(Point2{ x, .. self.position }, self.font_size, self.font_size);

            self.tiles.push(Tile::new(tilemap_index_from_char(c), rectangle));

            x += self.space_between_tiles;
        }
    }

    pub fn get_tiles(&self) -> &Vec<Tile> {
        &self.tiles
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }
}

impl GUIPosition for GUIText {
    fn width(&self) -> f32 { self.width }
    fn alignment(&self) -> GUIComponentAlignment {self.alignment}

    fn set_x(&mut self, x: f32) {
        self.position.x = x;

        let mut x = x;

        for tile in &mut self.tiles {
            let rectangle = GUIRectangle::new(Point2 {x, .. self.position}, self.font_size, self.font_size);
            tile.set_gui_rectangle(rectangle);

            x += self.space_between_tiles;
        }
    }

    fn calculate_component_position(&self, new_x: f32, width: f32) -> f32 {
        let x;

        let margin = 0.1;

        match self.alignment {
            GUIComponentAlignment::Left   => x = new_x + self.space_between_tiles/2.0 + margin,
            GUIComponentAlignment::Center => x = new_x - width/2.0 + self.space_between_tiles/2.0,
            GUIComponentAlignment::Right  => x = new_x - width + self.space_between_tiles/2.0 - margin,
        };

        x
    }

    fn update_position_from_half_screen_width(&mut self, width: f32) {
        match self.alignment() {
            GUIComponentAlignment::Left => self.update_component_position(-width),
            GUIComponentAlignment::Right => self.update_component_position(width),
            _ => (),
        }
    }
}


pub struct GUIFpsCounter {
    fps_text: GUIText,
    fps_count_text: GUIText,
    show_fps: bool,
}

impl GUIFpsCounter {
    pub fn new(x: f32, y: f32) -> GUIFpsCounter {
        let fps_text = GUIText::new_with_alignment(x, y, "FPS ", GUIComponentAlignment::Left);
        let fps_count_text = GUIText::new_with_alignment(x + fps_text.get_width(), y, "0", GUIComponentAlignment::Left);

        GUIFpsCounter {
            fps_text,
            fps_count_text,
            show_fps: false,
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

    pub fn update_position_from_half_screen_width(&mut self, width: f32) {
        self.fps_text.update_position_from_half_screen_width(width);
        self.fps_count_text.update_position_from_half_screen_width(width - self.fps_text.get_width());
    }
}


// TODO: Rename GUIHealthBar to GUISlider?

pub struct GUIHealthBar {
    rectangle: GUIRectangle<f32>,
    color: Vector3<f32>,
    alignment: GUIComponentAlignment,
    max_value: u32,
    low_value: u32,
    max_width: f32,
    x: f32,
    margin: f32,
    border_left: GUIRectangle<f32>,
    border_right: GUIRectangle<f32>,
    border_top: GUIRectangle<f32>,
    border_bottom: GUIRectangle<f32>,
    border_width: f32,
    change_color_when_low_value: bool,
}

impl GUIHealthBar {
    pub fn new(alignment: GUIComponentAlignment, x: f32, y: f32, max_width: f32, max_value: u32, low_value: u32, change_color_when_low_value: bool) -> GUIHealthBar {
        let margin;

        match alignment {
            GUIComponentAlignment::Left => margin = 0.2,
            GUIComponentAlignment::Right => margin =  -0.2,
            _ => margin = 0.0,
        }

        let border_width = 0.05;
        let border_height = 0.05;

        let height = 0.5;

        let health_bar = GUIHealthBar {
            rectangle: GUIRectangle::new(Point2::new(0.0,y),max_width,height),
            color: Vector3::zero(),
            alignment,
            max_value,
            low_value,
            max_width,
            x,
            margin,
            border_left: GUIRectangle::new(Point2::new(0.0, y), border_width, height + border_height*2.0),
            border_right: GUIRectangle::new(Point2::new(0.0, y), border_width, height + border_height*2.0),
            border_top: GUIRectangle::new(Point2::new(0.0, y + (height/2.0 + border_height/2.0)), max_width, border_height),
            border_bottom: GUIRectangle::new(Point2::new(0.0, y - (height/2.0 + border_height/2.0)), max_width, border_height),
            border_width,
            change_color_when_low_value,
        };
        health_bar
    }

    pub fn update_health(&mut self, health: u32) {
        if health <= self.low_value && self.change_color_when_low_value {
            self.color = Vector3::new(1.0,0.0,0.0);
        } else {
            self.color = Vector3::new(0.0,0.0,1.0);
        }

        if health > self.max_value {
            self.rectangle.set_width(self.max_width);
        } else {
            self.rectangle.set_width(self.max_width * (health as f32 / self.max_value as f32));
        }

        let x = self.x;
        self.update_component_position(x);
    }

    pub fn update_borders(&mut self) {
        let center_x;
        match self.alignment {
            GUIComponentAlignment::Left => {
                center_x = self.x + self.max_width/2.0;
            },
            GUIComponentAlignment::Right => {
                center_x = self.x - self.max_width/2.0;
            },
            GUIComponentAlignment::Center => center_x = self.x,
        }

        self.border_left.position_mut().x = center_x - self.max_width/2.0 - self.border_width/2.0;
        self.border_left.update_model_matrix();

        self.border_right.position_mut().x = center_x + self.max_width/2.0 + self.border_width/2.0;
        self.border_right.update_model_matrix();

        self.border_top.position_mut().x = center_x;
        self.border_top.update_model_matrix();

        self.border_bottom.position_mut().x = center_x;
        self.border_bottom.update_model_matrix();

    }

    pub fn borders(&self) -> [&GUIRectangle<f32>; 4] {
        [
            &self.border_left,
            &self.border_right,
            &self.border_top,
            &self.border_bottom,
        ]
    }
}

impl_model_matrix!(GUIHealthBar, rectangle);
impl_color!(GUIHealthBar);


impl GUIPosition for GUIHealthBar {
    fn width(&self) -> f32 { self.rectangle.width() }
    fn alignment(&self) -> GUIComponentAlignment { self.alignment }
    fn set_x(&mut self, x: f32) {
        self.rectangle.position_mut().x = x;
        self.rectangle.update_model_matrix();
    }

    fn update_position_from_half_screen_width(&mut self, width: f32) {
        match self.alignment() {
            GUIComponentAlignment::Left => {
                self.x = -width + self.margin;
                let x = self.x;
                self.update_component_position(x)
            },
            GUIComponentAlignment::Right => {
                self.x = width + self.margin;
                let x = self.x;
                self.update_component_position(x)
            },
            _ => (),
        }
        self.update_borders();
    }
}