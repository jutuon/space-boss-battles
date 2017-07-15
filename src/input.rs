/*
src/input.rs, 2017-07-15

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use sdl2::keyboard::Keycode;


pub struct InputKeyboard {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl InputKeyboard {
    pub fn new() -> InputKeyboard {
        InputKeyboard {up: false, down: false, left: false, right: false}
    }

    pub fn update_key_up(&mut self, key: Keycode) {
        self.update_keys(key, false);
    }

    pub fn update_key_down(&mut self, key: Keycode) {
        self.update_keys(key, true);
    }

    fn update_keys(&mut self, key: Keycode, value: bool) {
        match key {
            Keycode::Up => self.up = value,
            Keycode::Down => self.down = value,
            Keycode::Left => self.left = value,
            Keycode::Right => self.right = value,
            _ => (),
        }
    }
}

pub trait Input {
    fn up(&self) -> bool;
    fn down(&self) -> bool;
    fn left(&self) -> bool;
    fn right(&self) -> bool;
}

impl Input for InputKeyboard {
    fn up(&self) -> bool { self.up }
    fn down(&self) -> bool { self.down }
    fn left(&self) -> bool { self.left }
    fn right(&self) -> bool { self.right }
}