/*
src/input.rs, 2017-07-30

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use sdl2::keyboard::Keycode;

use cgmath::Point2;

use Timer;
use time::PreciseTime;

pub struct InputKeyboard {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    shoot: bool,

    key_hit_left: bool,
    key_hit_right: bool,
    key_hit_enter: bool,
    key_hit_back: bool,

    key_hit_up_timer: KeyHitTimer,
    key_hit_down_timer: KeyHitTimer,

    mouse_motion: bool,
    mouse_button_hit: bool,
    mouse_location: Point2<f32>
}

impl InputKeyboard {
    pub fn new() -> InputKeyboard {
        InputKeyboard {
            up: false,
            down: false,
            left: false,
            right: false,
            shoot: false,

            key_hit_left: false,
            key_hit_right: false,
            key_hit_enter: false,
            key_hit_back: false,

            key_hit_up_timer: KeyHitTimer::new(Keycode::Up),
            key_hit_down_timer: KeyHitTimer::new(Keycode::Down),

            mouse_motion: true,
            mouse_button_hit: true,
            mouse_location: Point2::new(0.0, 0.0),
        }
    }

    pub fn reset_key_hits(&mut self) {
        let value = false;
        self.key_hit_left = value;
        self.key_hit_right = value;
        self.key_hit_enter = value;
        self.key_hit_back = value;

        self.key_hit_up_timer.reset();
        self.key_hit_down_timer.reset();

        self.mouse_button_hit = false;
    }

    pub fn update_key_up(&mut self, key: Keycode) {
        self.update_keys(key, false);
        self.update_key_hit(key, true);

        self.key_hit_up_timer.event_key_up(key);
        self.key_hit_down_timer.event_key_up(key);
    }

    pub fn update_key_down(&mut self, key: Keycode) {
        self.update_keys(key, true);

        let current_time = PreciseTime::now();
        self.key_hit_up_timer.event_key_down(key, current_time);
        self.key_hit_down_timer.event_key_down(key, current_time);
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

    fn update_key_hit(&mut self, key: Keycode, value: bool) {
        match key {
            Keycode::Left       => self.key_hit_left = value,
            Keycode::Right      => self.key_hit_right = value,
            Keycode::Return     => self.key_hit_enter = value,
            Keycode::Backspace  => self.key_hit_back = value,
            _ => (),
        }
    }

    pub fn update_mouse_motion(&mut self, point: Point2<f32>) {
        self.mouse_motion = true;
        self.mouse_location = point;
    }

    pub fn update_mouse_button_up(&mut self, point: Point2<f32>) {
        self.mouse_button_hit = true;
        self.mouse_location = point;
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

    fn key_hit_up(&mut self) -> bool;
    fn key_hit_down(&mut self) -> bool;
    fn key_hit_left(&mut self) -> bool;
    fn key_hit_right(&mut self) -> bool;
    fn key_hit_enter(&mut self) -> bool;
    fn key_hit_back(&mut self) -> bool;

    fn mouse_button_hit(&mut self) -> bool;
    fn mouse_motion(&mut self) -> bool;
    fn mouse_location(&self) -> &Point2<f32>;
}

impl Input for InputKeyboard {
    fn up(&self) -> bool    { self.up    }
    fn down(&self) -> bool  { self.down  }
    fn left(&self) -> bool  { self.left  }
    fn right(&self) -> bool { self.right }
    fn shoot(&self) -> bool { self.shoot }

    fn key_hit_up(&mut self) -> bool     { self.key_hit_up_timer.return_and_reset()    }
    fn key_hit_down(&mut self) -> bool   { self.key_hit_down_timer.return_and_reset()  }
    fn key_hit_left(&mut self) -> bool   { return_and_reset(&mut self.key_hit_left)  }
    fn key_hit_right(&mut self) -> bool  { return_and_reset(&mut self.key_hit_right) }
    fn key_hit_enter(&mut self) -> bool  { return_and_reset(&mut self.key_hit_enter) }
    fn key_hit_back(&mut self) -> bool   { return_and_reset(&mut self.key_hit_back) }

    fn mouse_button_hit(&mut self) -> bool      { return_and_reset(&mut self.mouse_button_hit) }
    fn mouse_motion(&mut self) -> bool          { return_and_reset(&mut self.mouse_motion) }
    fn mouse_location(&self) -> &Point2<f32>    { &self.mouse_location }
}

enum KeyHitState {
    NormalMode,
    ScrollMode,
}

struct KeyHitTimer {
    timer: Timer,
    state: Option<KeyHitState>,
    key: Keycode,
    key_hit: bool,
}

impl KeyHitTimer {
    fn new(key: Keycode) -> KeyHitTimer {
        KeyHitTimer {
            timer: Timer::new(),
            state: None,
            key: key,
            key_hit: false,
        }
    }

    fn event_key_down(&mut self, key: Keycode, current_time: PreciseTime) {
        if self.key != key {
            return;
        }

        match self.state {
            None => {
                self.state = Some(KeyHitState::NormalMode);
                self.timer.reset(current_time);
            },
            Some(KeyHitState::NormalMode) => {
                if self.timer.check(current_time, 400) {
                    self.state = Some(KeyHitState::ScrollMode);
                    self.key_hit = true;
                }
            },
            Some(KeyHitState::ScrollMode) => {
                if self.timer.check(current_time, 400) {
                    self.key_hit = true;
                }
            }
        }
    }

    fn event_key_up(&mut self, key: Keycode) {
        if self.key != key {
            return;
        }

        match self.state {
            Some(KeyHitState::NormalMode) => {
                self.key_hit = true;
            },
            _ => {
                self.key_hit = false;
                self.state = None;
            },
        }
    }

    fn return_and_reset(&mut self) -> bool {
        if self.key_hit {
            self.key_hit = false;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.key_hit = false;
    }
}