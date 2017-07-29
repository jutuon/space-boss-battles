/*
src/input.rs, 2017-07-29

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use sdl2::keyboard::Keycode;

use Timer;
use time::PreciseTime;

pub struct InputKeyboard {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    shoot: bool,

    keyhit_left: bool,
    keyhit_right: bool,
    keyhit_enter: bool,
    keyhit_back: bool,

    keyhit_up_timer: KeyhitTimer,
    keyhit_down_timer: KeyhitTimer,
}

impl InputKeyboard {
    pub fn new() -> InputKeyboard {
        InputKeyboard {
            up: false,
            down: false,
            left: false,
            right: false,
            shoot: false,

            keyhit_left: false,
            keyhit_right: false,
            keyhit_enter: false,
            keyhit_back: false,

            keyhit_up_timer: KeyhitTimer::new(Keycode::Up),
            keyhit_down_timer: KeyhitTimer::new(Keycode::Down),
        }
    }

    pub fn reset_keyhits(&mut self) {
        let value = false;
        self.keyhit_left = value;
        self.keyhit_right = value;
        self.keyhit_enter = value;
        self.keyhit_back = value;

        self.keyhit_up_timer.reset();
        self.keyhit_down_timer.reset();
    }

    pub fn update_key_up(&mut self, key: Keycode) {
        self.update_keys(key, false);
        self.update_keyhit(key, true);

        self.keyhit_up_timer.event_key_up(key);
        self.keyhit_down_timer.event_key_up(key);
    }

    pub fn update_key_down(&mut self, key: Keycode) {
        self.update_keys(key, true);

        let current_time = PreciseTime::now();
        self.keyhit_up_timer.event_key_down(key, current_time);
        self.keyhit_down_timer.event_key_down(key, current_time);
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

    fn keyhit_up(&mut self) -> bool     { self.keyhit_up_timer.return_and_reset()    }
    fn keyhit_down(&mut self) -> bool   { self.keyhit_down_timer.return_and_reset()  }
    fn keyhit_left(&mut self) -> bool   { return_and_reset(&mut self.keyhit_left)  }
    fn keyhit_right(&mut self) -> bool  { return_and_reset(&mut self.keyhit_right) }
    fn keyhit_enter(&mut self) -> bool  { return_and_reset(&mut self.keyhit_enter) }
    fn keyhit_back(&mut self) -> bool   { return_and_reset(&mut self.keyhit_back) }
}

enum KeyhitState {
    NormalMode,
    ScrollMode,
}

struct KeyhitTimer {
    timer: Timer,
    state: Option<KeyhitState>,
    key: Keycode,
    keyhit: bool,
}

impl KeyhitTimer {
    fn new(key: Keycode) -> KeyhitTimer {
        KeyhitTimer {
            timer: Timer::new(),
            state: None,
            key: key,
            keyhit: false,
        }
    }

    fn event_key_down(&mut self, key: Keycode, current_time: PreciseTime) {
        if self.key != key {
            return;
        }

        match self.state {
            None => {
                self.state = Some(KeyhitState::NormalMode);
                self.timer.reset(current_time);
            },
            Some(KeyhitState::NormalMode) => {
                if self.timer.check(current_time, 400) {
                    self.state = Some(KeyhitState::ScrollMode);
                    self.keyhit = true;
                }
            },
            Some(KeyhitState::ScrollMode) => {
                if self.timer.check(current_time, 400) {
                    self.keyhit = true;
                }
            }
        }
    }

    fn event_key_up(&mut self, key: Keycode) {
        if self.key != key {
            return;
        }

        match self.state {
            Some(KeyhitState::NormalMode) => {
                self.keyhit = true;
            },
            _ => {
                self.keyhit = false;
                self.state = None;
            },
        }
    }

    fn return_and_reset(&mut self) -> bool {
        if self.keyhit {
            self.keyhit = false;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.keyhit = false;
    }
}