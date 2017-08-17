/*
src/gui/components.rs, 2017-08-17

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

const GUI_HEALTH_BAR_LEFT_AND_RIGHT_MARGIN: f32 = 0.2;
const GUI_HEALTH_BAR_BORDER_WIDTH: f32 = 0.05;
const GUI_HEALTH_BAR_BORDER_HEIGHT: f32 = 0.05;
const GUI_HEALTH_BAR_HEIGHT_NOT_INCLUDING_BORDERS: f32 = 0.5;

const GUI_HEALTH_BAR_LOW_VALUE_COLOR: Vector3<f32> = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
const GUI_HEALTH_BAR_COLOR: Vector3<f32> = Vector3 { x: 0.0, y: 0.0, z: 1.0 };

const GUI_BUTTON_COLOR:  Vector3<f32> = Vector3 { x: 0.0, y: 0.0, z: 0.4 };
const GUI_BUTTON_SELECTED_COLOR:  Vector3<f32> = Vector3 { x: 0.0, y: 0.0, z: 1.0 };


const GUI_TEXT_MARGIN_LEFT_RIGHT: f32 = 0.1;


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


/// Collision detection, state setting and event saving for components
/// providing user interaction.
pub trait GUIUserInteraction {
    /// If point is inside the component area, return true.
    fn collision(&self, point: &Point2<f32>) -> bool;
    /// Set new state to component.
    fn set_state(&mut self, state: GUIComponentState);
    /// Get event data.
    fn event_data(&self) -> GUIEvent;
    /// Set event data.
    fn set_event_data(&mut self, data: GUIEvent);
}

/// Position updates and calculations for components
/// with alignment.
pub trait GUIPosition {
    /// Updates position from argument `width_half` which is
    /// screen_width/2.0.
    fn update_position_from_half_screen_width(&mut self, width_half: f32);
    /// Component width.
    fn width(&self) -> f32;
    /// Set component x position.
    fn set_x(&mut self, x: f32);
    /// Get current alignment setting.
    fn alignment(&self) -> GUIComponentAlignment;

    /// Calculate and return new x position for component.
    ///
    /// Component width and alignment is used to perform the position calculation.
    ///
    /// # Arguments
    /// * `new_x` is x coordinate where user wants to position the component.
    fn calculate_component_position(&self, new_x: f32) -> f32 {
        let mut x = new_x;

        let half_width = self.width()/2.0;

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

    /// Calculates and sets new x position to component.
    fn update_component_position(&mut self, new_x: f32) {
        let x = self.calculate_component_position(new_x);
        self.set_x(x);
    }
}

/// State of component which implements `GUIUserInteraction` trait.
pub enum GUIComponentState {
    Selected,
    Normal,
}

/// Component alignment.
#[derive(Copy, Clone)]
pub enum GUIComponentAlignment {
    Left,
    Right,
    Center,
}

/// Geometric primitive GUI component which can be rendered.
/// All other components are based on this.
pub struct GUIRectangle<T> {
    model_matrix: Matrix4<T>,
    position: Point2<T>,
    width: T,
    height: T,
}

impl GUIRectangle<f32> {
    /// Create new `GUIRectangle<f32>`.
    ///
    /// Updates the model matrix of created rectangle.
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

    /// Updates rectangle's model matrix.
    fn update_model_matrix(&mut self) {
        self.model_matrix = Matrix4::from_nonuniform_scale(self.width, self.height, 1.0);

        self.model_matrix.w.x = self.position.x;
        self.model_matrix.w.y = self.position.y;
    }

    /// Checks if there is collision between point and rectangle. Argument `point` must be in world coordinates.
    ///
    /// Note that you can't rotate `GUIRectangle` so axis aligned collision check will work nicely.
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

    /// Get position as mutable reference.
    ///
    /// Remember to update model matrix after changing the position.
    fn position_mut(&mut self) -> &mut Point2<f32> {
        &mut self.position
    }

    /// Set width.
    ///
    /// Remember to update model matrix after changing the width.
    fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    /// Get width.
    fn width(&self) -> f32 {
        self.width
    }
}

impl_model_matrix!(GUIRectangle<f32>);


/// Button with text.
pub struct GUIButton {
    rectangle: GUIRectangle<f32>,
    text: GUIText,
    color: Vector3<f32>,
    event_data: GUIEvent,
}

impl GUIButton {
    /// Creates new `GUIButton`.
    ///
    /// Argument `event_data` is event what will happen when button is pressed.
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: &str, event_data: GUIEvent) -> GUIButton {
        let mut button = GUIButton {
            rectangle: GUIRectangle::new(Point2 {x, y}, width, height),
            text: GUIText::new(x, y, text),
            color: Vector3::zero(),
            event_data,
        };

        button.set_state(GUIComponentState::Normal);

        button
    }

    /// Get button's `GUIText`.
    pub fn get_text(&self) -> &GUIText {
        &self.text
    }
}

impl_model_matrix!(GUIButton, rectangle);
impl_color!(GUIButton);


impl GUIUserInteraction for GUIButton {
    fn collision(&self, point: &Point2<f32>) -> bool {
        self.rectangle.axis_aligned_rectangle_and_point_collision(point)
    }

    /// Sets button color according to argument `state`.
    fn set_state(&mut self, state: GUIComponentState) {
        match state {
            GUIComponentState::Normal => self.color = GUI_BUTTON_COLOR,
            GUIComponentState::Selected => self.color = GUI_BUTTON_SELECTED_COLOR,
        }
    }

    fn event_data(&self) -> GUIEvent {
        self.event_data
    }

    fn set_event_data(&mut self, data: GUIEvent) {
        self.event_data = data;
    }
}

/// Builds non empty `GUIGroups`.
pub struct GUIGroupBuilder<T: GUIUserInteraction> {
    components: Vec<T>,
}

impl <T: GUIUserInteraction> GUIGroupBuilder<T> {
    /// Create new `GUIGroupBuilder<T>`.
    pub fn new() -> GUIGroupBuilder<T> {
        GUIGroupBuilder {
            components: Vec::new(),
        }
    }

    /// Add GUI component.
    pub fn add(&mut self, gui_component: T) {
        self.components.push(gui_component);
    }

    /// Create `GUIGroup<T>`
    ///
    /// Sets first GUI component selected.
    ///
    /// # Panics
    /// If `GUIGroupBuilder<T>` is empty.
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

/// Handles current selection between GUI components which implements `GUIUserInteraction`
/// trait.
pub struct GUIGroup<T: GUIUserInteraction> {
    components: Vec<T>,
    selected: usize,
}

impl <T: GUIUserInteraction> GUIGroup<T> {
    /// Create new `GUIGroup<T>`.
    ///
    /// Sets `first_component`'s state as selected.
    pub fn new(mut first_component: T) -> GUIGroup<T> {
        first_component.set_state(GUIComponentState::Selected);

        let mut vec = Vec::new();
        vec.push(first_component);

        GUIGroup {
            components: vec,
            selected: 0,
        }
    }

    /// Adds next component to `GUIGroup`.
    pub fn add(mut self, component: T) -> GUIGroup<T> {
        self.components.push(component);
        self
    }

    /// Move selection up.
    ///
    /// The selection will move to the last component if current selection is
    /// the first component.
    pub fn selection_up(&mut self) {
        self.update_selection_index(true);
    }

    /// Move selection down.
    ///
    /// The selection will move to the first component if current selection is
    /// the last component.
    pub fn selection_down(&mut self) {
        self.update_selection_index(false);
    }

    /// Updates selection index and states of components which need updating because of selection
    /// index change.
    ///
    /// The direction where index will go is set by `selection_up` argument.
    ///
    /// The selection will move to the last component if current selection is
    /// the first component.
    ///
    /// The selection will move to the first component if current selection is
    /// the last component.
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

    /// Get components.
    pub fn get_components(&self) -> &[T] {
        self.components.as_slice()
    }

    /// Get components with mutability.
    pub fn get_components_mut(&mut self) -> &mut [T] {
        self.components.as_mut_slice()
    }

    /// Updates selection to that component where collision is detected.
    ///
    /// Argument `point` must be in world coordinates.
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

    /// Sets new event to currently selected component.
    pub fn set_event_of_currently_selected_component(&mut self, event: GUIEvent) {
        self.components[self.selected].set_event_data(event);
    }

    /// Get event of currently selected component.
    pub fn event_of_currently_selected_component(&self) -> GUIEvent {
        self.components[self.selected].event_data()
    }

    /// Check collision and return event of that component where collision was.
    pub fn check_collision_and_return_event(&self, point: &Point2<f32>) -> Option<GUIEvent> {
        for button in &self.components {
            if button.collision(point) {
                return Some(button.event_data());
            }
        };

        None
    }
}


/// Tile from tile map.
///
/// For information about `tile_info` field, check renderer documentation.
pub struct Tile {
    rectangle: GUIRectangle<f32>,
    tile_info: Vector3<f32>,
}

impl Tile {
    /// Create new `Tile`.
    ///
    /// This will assume that tile size is 16x16 pixels and tile map size is 256x256 pixels.
    ///
    /// # Index argument
    /// * Starts from top left corner with (0,0);
    pub fn new((x, y): (u32, u32), gui_rectangle: GUIRectangle<f32>) -> Tile {
        let tile_size = 1.0/16.0;
        let x_movement = tile_size * x as f32;
        let y_movement = 1.0 - tile_size * (y + 1) as f32;

        Tile {
            rectangle: gui_rectangle,
            tile_info: Vector3::new(x_movement, y_movement, tile_size),
        }
    }

    /// Sets new `GUIRectangle` for a `Tile`. This means setting the location
    /// where tile will be rendered.
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

/// Convert char to position index at game's font tile map.
///
/// First item in tuple will be x index, and second item will be y index.
fn tile_map_index_from_char(c: char) -> (u32, u32) {
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

        _ => tile_map_index_from_char(' '),

    }
}

/// Text for GUI.
///
/// Text will be rendered as tiles from tile map font.
pub struct GUIText {
    tiles: Vec<Tile>,
    position: Point2<f32>,
    font_size: f32,
    tile_width: f32,
    width: f32,
    alignment: GUIComponentAlignment,
}

impl GUIText {
    /// Creates new text with `GUIComponentAlignment::Center`.
    pub fn new(x: f32, y: f32, text: &str) -> GUIText {
        GUIText::new_with_alignment(x, y, text, GUIComponentAlignment::Center)
    }

    /// Create new text.
    pub fn new_with_alignment(x: f32, y: f32, text: &str, alignment: GUIComponentAlignment) -> GUIText {
        let mut gui_text = GUIText {
            tiles: Vec::new(),
            // Add little offset in y direction to make text look centered
            // in y direction, because in the current tile map font, the letters are not in center.
            position: Point2 {x, y: y - 0.04},
            font_size: 0.57,
            tile_width: 0.0,
            width: 0.0,
            alignment,
        };

        gui_text.change_text(text);

        gui_text
    }

    /// Update `GUIText` to have a new text.
    pub fn change_text(&mut self, text: &str) {
        self.tiles.clear();

        let text_len = text.len() as f32;

        self.tile_width = self.font_size - 0.17;
        self.width = text_len * self.tile_width;

        let mut x = self.calculate_component_position(self.position.x);

        for c in text.chars() {
            let rectangle = GUIRectangle::new(Point2{ x, .. self.position }, self.font_size, self.font_size);

            self.tiles.push(Tile::new(tile_map_index_from_char(c), rectangle));

            x += self.tile_width;
        }
    }

    /// Get tiles.
    pub fn get_tiles(&self) -> &Vec<Tile> {
        &self.tiles
    }
}

impl GUIPosition for GUIText {
    /// Text width.
    fn width(&self) -> f32 { self.width }
    fn alignment(&self) -> GUIComponentAlignment {self.alignment}

    fn set_x(&mut self, x: f32) {
        self.position.x = x;

        let mut x = x;

        for tile in &mut self.tiles {
            let rectangle = GUIRectangle::new(Point2 {x, .. self.position}, self.font_size, self.font_size);
            tile.set_gui_rectangle(rectangle);

            x += self.tile_width;
        }
    }

    fn calculate_component_position(&self, new_x: f32) -> f32 {
        let x;

        match self.alignment {
            GUIComponentAlignment::Left   => x = new_x + self.tile_width/2.0 + GUI_TEXT_MARGIN_LEFT_RIGHT,
            GUIComponentAlignment::Center => x = new_x - self.width/2.0 + self.tile_width/2.0,
            GUIComponentAlignment::Right  => x = new_x - self.width + self.tile_width/2.0 - GUI_TEXT_MARGIN_LEFT_RIGHT,
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


/// FPS counter positioned to the left side of the screen.
pub struct GUIFpsCounter {
    fps_text: GUIText,
    fps_count_text: GUIText,
    show_fps: bool,
}

impl GUIFpsCounter {
    /// Create new `GUIFpsCounter`
    pub fn new(x: f32, y: f32) -> GUIFpsCounter {
        let fps_text = GUIText::new_with_alignment(x, y, "FPS ", GUIComponentAlignment::Left);
        let fps_count_text = GUIText::new_with_alignment(x + fps_text.width(), y, "0", GUIComponentAlignment::Left);

        GUIFpsCounter {
            fps_text,
            fps_count_text,
            show_fps: false,
        }
    }

    /// Set new fps count.
    pub fn update_fps_count(&mut self, fps_count: u32) {
        let text = fps_count.to_string();
        self.fps_count_text.change_text(&text);
    }

    /// Get texts of `GUIFpsCounter`.
    pub fn texts(&self) -> [&GUIText; 2] {
        [&self.fps_text, &self.fps_count_text]
    }

    /// Get fps counter visibility.
    pub fn show_fps(&self) -> bool {
        self.show_fps
    }

    /// Set fps counter visibility.
    pub fn set_show_fps(&mut self, value: bool) {
        self.show_fps = value;
    }

    /// Update fps counter position.
    ///
    /// Argument `width` is screen_width/2.0.
    pub fn update_position_from_half_screen_width(&mut self, width: f32) {
        self.fps_text.update_position_from_half_screen_width(width);
        self.fps_count_text.update_position_from_half_screen_width(width - self.fps_text.width());
    }
}


// TODO: Rename GUIHealthBar to GUISlider?

/// Graphical value indicator.
pub struct GUIHealthBar {
    rectangle: GUIRectangle<f32>,
    color: Vector3<f32>,
    border_color: Vector3<f32>,
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
    /// Create new `GUIHealthBar`.
    pub fn new(alignment: GUIComponentAlignment, x: f32, y: f32, max_width: f32, max_value: u32, low_value: u32, change_color_when_low_value: bool) -> GUIHealthBar {
        let margin = match alignment {
            GUIComponentAlignment::Left => GUI_HEALTH_BAR_LEFT_AND_RIGHT_MARGIN,
            GUIComponentAlignment::Right => -GUI_HEALTH_BAR_LEFT_AND_RIGHT_MARGIN,
            _ => 0.0,
        };

        let mut health_bar = GUIHealthBar {
            rectangle: GUIRectangle::new(Point2::new(0.0, y), max_width, GUI_HEALTH_BAR_HEIGHT_NOT_INCLUDING_BORDERS),
            color: Vector3::zero(),
            border_color: Vector3::zero(),
            alignment,
            max_value,
            low_value,
            max_width,
            x,
            margin,
            border_left: GUIRectangle::new(Point2::new(0.0, y), GUI_HEALTH_BAR_BORDER_WIDTH, GUI_HEALTH_BAR_HEIGHT_NOT_INCLUDING_BORDERS + GUI_HEALTH_BAR_BORDER_HEIGHT*2.0),
            border_right: GUIRectangle::new(Point2::new(0.0, y), GUI_HEALTH_BAR_BORDER_WIDTH, GUI_HEALTH_BAR_HEIGHT_NOT_INCLUDING_BORDERS + GUI_HEALTH_BAR_BORDER_HEIGHT*2.0),
            border_top: GUIRectangle::new(Point2::new(0.0, y + (GUI_HEALTH_BAR_HEIGHT_NOT_INCLUDING_BORDERS/2.0 + GUI_HEALTH_BAR_BORDER_HEIGHT/2.0)), max_width, GUI_HEALTH_BAR_BORDER_HEIGHT),
            border_bottom: GUIRectangle::new(Point2::new(0.0, y - (GUI_HEALTH_BAR_HEIGHT_NOT_INCLUDING_BORDERS/2.0 + GUI_HEALTH_BAR_BORDER_HEIGHT/2.0)), max_width, GUI_HEALTH_BAR_BORDER_HEIGHT),
            border_width: GUI_HEALTH_BAR_BORDER_WIDTH,
            change_color_when_low_value,
        };

        health_bar.update_borders();

        if let GUIComponentAlignment::Center = health_bar.alignment {
            health_bar.x -= max_width/2.0;
            health_bar.alignment = GUIComponentAlignment::Left;
        }

        health_bar
    }

    /// Updates health bar's visual appearance according to new health value.
    pub fn update_health(&mut self, health: u32) {
        if health <= self.low_value && self.change_color_when_low_value {
            self.color = GUI_HEALTH_BAR_LOW_VALUE_COLOR;
            self.border_color = GUI_HEALTH_BAR_LOW_VALUE_COLOR;
        } else {
            self.color = GUI_HEALTH_BAR_COLOR;
            self.border_color = GUI_HEALTH_BAR_COLOR;
        }

        if health > self.max_value {
            self.rectangle.set_width(self.max_width);
        } else {
            self.rectangle.set_width(self.max_width * (health as f32 / self.max_value as f32));
        }

        let x = self.x;
        self.update_component_position(x);
    }

    /// Updates border positions.
    pub fn update_borders(&mut self) {
        let center_x = match self.alignment {
            GUIComponentAlignment::Left => {
                self.x + self.max_width/2.0
            },
            GUIComponentAlignment::Right => {
                self.x - self.max_width/2.0
            },
            GUIComponentAlignment::Center => self.x,
        };

        self.border_left.position_mut().x = center_x - self.max_width/2.0 - self.border_width/2.0;
        self.border_left.update_model_matrix();

        self.border_right.position_mut().x = center_x + self.max_width/2.0 + self.border_width/2.0;
        self.border_right.update_model_matrix();

        self.border_top.position_mut().x = center_x;
        self.border_top.update_model_matrix();

        self.border_bottom.position_mut().x = center_x;
        self.border_bottom.update_model_matrix();

    }

    /// Get border references.
    pub fn borders(&self) -> [&GUIRectangle<f32>; 4] {
        [
            &self.border_left,
            &self.border_right,
            &self.border_top,
            &self.border_bottom,
        ]
    }

    pub fn border_color(&self) -> &Vector3<f32> {
        &self.border_color
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