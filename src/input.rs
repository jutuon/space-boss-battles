/*
src/input.rs, 2017-07-28

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
    shoot: bool,

    keyhit_up: bool,
    keyhit_down: bool,
    keyhit_left: bool,
    keyhit_right: bool,
    keyhit_enter: bool,
    keyhit_back: bool,
}

impl InputKeyboard {
    pub fn new() -> InputKeyboard {
        InputKeyboard {
            up: false,
            down: false,
            left: false,
            right: false,
            shoot: false,

            keyhit_up: false,
            keyhit_down: false,
            keyhit_left: false,
            keyhit_right: false,
            keyhit_enter: false,
            keyhit_back: false,
        }
    }

    pub fn reset_keyhits(&mut self) {
        let value = false;
        self.keyhit_up = value;
        self.keyhit_down = value;
        self.keyhit_left = value;
        self.keyhit_right = value;
        self.keyhit_enter = value;
        self.keyhit_back = value;
    }

    pub fn update_key_up(&mut self, key: Keycode) {
        self.update_keys(key, false);
        self.update_keyhit(key, true);
    }

    pub fn update_key_down(&mut self, key: Keycode) {
        self.update_keys(key, true);
        self.update_keyhit(key, false);
    }

    fn update_keys(&mut self, key: Keycode, value: bool) {
        match key {
            Keycode::Up     => self.up = value,
            Keycode::Down   => self.down = value,
            Keycode::Left   => self.left = value,
            Keycode::Right  => self.right = value,
            Keycode::Space  => self.shoot = value,
            _ => (),
        }
    }

    fn update_keyhit(&mut self, key: Keycode, value: bool) {
        match key {
            Keycode::Up         => self.keyhit_up = value,
            Keycode::Down       => self.keyhit_down = value,
            Keycode::Left       => self.keyhit_left = value,
            Keycode::Right      => self.keyhit_right = value,
            Keycode::Return     => self.keyhit_enter = value,
            Keycode::Backspace  => self.keyhit_back = value,
            _ => (),
        }
    }
}

fn return_and_reset(value: &mut bool) -> bool {
    let original_value: bool = *value;
    *value = false;
    original_value
}

pub trait Input {
    fn up(&self) -> bool;
    fn down(&self) -> bool;
    fn left(&self) -> bool;
    fn right(&self) -> bool;
    fn shoot(&self) -> bool;

    fn keyhit_up(&mut self) -> bool;
    fn keyhit_down(&mut self) -> bool;
    fn keyhit_left(&mut self) -> bool;
    fn keyhit_right(&mut self) -> bool;
    fn keyhit_enter(&mut self) -> bool;
    fn keyhit_back(&mut self) -> bool;
}

impl Input for InputKeyboard {
    fn up(&self) -> bool    { self.up    }
    fn down(&self) -> bool  { self.down  }
    fn left(&self) -> bool  { self.left  }
    fn right(&self) -> bool { self.right }
    fn shoot(&self) -> bool { self.shoot }

    fn keyhit_up(&mut self) -> bool     { return_and_reset(&mut self.keyhit_up)    }
    fn keyhit_down(&mut self) -> bool   { return_and_reset(&mut self.keyhit_down)  }
    fn keyhit_left(&mut self) -> bool   { return_and_reset(&mut self.keyhit_left)  }
    fn keyhit_right(&mut self) -> bool  { return_and_reset(&mut self.keyhit_right) }
    fn keyhit_enter(&mut self) -> bool  { return_and_reset(&mut self.keyhit_enter) }
    fn keyhit_back(&mut self) -> bool   { return_and_reset(&mut self.keyhit_back) }
}