/*
src/input.rs, 2017-08-06

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use sdl2::keyboard::Keycode;
use sdl2::{GameControllerSubsystem, JoystickSubsystem};
use sdl2::controller::{GameController, Button, Axis};

use cgmath::Point2;

use Timer;
use time::PreciseTime;

pub struct InputManager {
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
    mouse_location: Point2<f32>,

    game_controller_subsystem: GameControllerSubsystem,
    game_controllers: Vec<GameController>,

    joystick_subsystem: JoystickSubsystem,
}

impl InputManager {
    pub fn new(game_controller_subsystem: GameControllerSubsystem, joystick_subsystem: JoystickSubsystem) -> InputManager {
        InputManager {
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

            game_controller_subsystem: game_controller_subsystem,
            game_controllers: Vec::new(),

            joystick_subsystem: joystick_subsystem,
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

    pub fn add_game_controller(&mut self, id: u32) {
        if self.game_controller_subsystem.is_game_controller(id) {
            match self.game_controller_subsystem.open(id) {
                Ok(controller) => {
                    self.game_controllers.push(controller);
                    println!("game controller with id {} added", id);
                },
                Err(integer_or_sdl_error) => println!("game controller error: {}", integer_or_sdl_error),
            }
        }
    }

    pub fn add_joystick(&mut self, id: u32) {
        if !self.game_controller_subsystem.is_game_controller(id) {
            let joystick_name;
            match self.joystick_subsystem.name_for_index(id) {
                Ok(name) => joystick_name = name,
                Err(error) => {
                    println!("error: {}", error);
                    return;
                }
            }

            let mut joystick_guid;
            match self.joystick_subsystem.device_guid(id) {
                Ok(guid) => joystick_guid = guid.to_string(),
                Err(error) => {
                    println!("error: {}", error);
                    return;
                }
            }

            // https://wiki.libsdl.org/SDL_GameControllerAddMapping
            joystick_guid.push(',');
            joystick_guid.push_str(&joystick_name);
            joystick_guid.push_str(", a:b2, b:b1, y:b0, x:b3, start:b9, guide:b12, back:b8, dpup:h0.1, dpleft:h0.8, dpdown:h0.4, dpright:h0.2, leftshoulder:b6, rightshoulder:b7, leftstick:b10, rightstick:b11, leftx:a0, lefty:a1, rightx:a3, righty:a2, lefttrigger:b4, righttrigger:b5");

            match self.game_controller_subsystem.add_mapping(&joystick_guid) {
                Ok(_) => {
                    println!("default game controller mapping loaded for joystick with id {}", id);
                    self.add_game_controller(id)
                },
                Err(error) => println!("error: {}", error),
            }
        }
    }

    pub fn remove_game_controller(&mut self, id: i32) {
        let mut index = None;

        for (i, controller) in self.game_controllers.iter().enumerate() {
            if controller.instance_id() == id {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            self.game_controllers.swap_remove(i);
        }

        println!("game controller with id {} removed", id);
    }

    pub fn game_controller_button_up(&mut self, button: Button) {
        self.update_game_controller_buttons(button, false);
        self.update_key_hit_values_from_game_controller(button, true);

        if let Button::DPadUp = button {
            self.key_hit_up_timer.event_key_up(Keycode::Up);
        } else if let Button::DPadDown = button {
            self.key_hit_down_timer.event_key_up(Keycode::Down);
        }
    }

    pub fn game_controller_button_down(&mut self, button: Button) {
        self.update_game_controller_buttons(button, true);

        let current_time = PreciseTime::now();
        if let Button::DPadUp = button {
            self.key_hit_up_timer.event_key_down(Keycode::Up, current_time);
        } else if let Button::DPadDown = button {
            self.key_hit_down_timer.event_key_down(Keycode::Down, current_time);
        }
    }

    pub fn game_controller_axis_motion(&mut self, axis: Axis, value: i16) {
        match axis {
            Axis::LeftX | Axis::RightX => {
                if value > 10000 {
                    self.right = true;
                } else if value < -10000 {
                    self.left = true;
                } else {
                    self.left = false;
                    self.right = false;
                }
            },
            Axis::LeftY | Axis::RightY => {
                let current_time = PreciseTime::now();

                if value > 10000 {
                    self.down = true;
                    self.key_hit_down_timer.event_key_down_scroll_mode_only(Keycode::Down, current_time);

                } else if value < -10000 {
                    self.up = true;
                    self.key_hit_up_timer.event_key_down_scroll_mode_only(Keycode::Up, current_time);
                } else {
                    self.up = false;
                    self.down = false;
                    self.key_hit_down_timer.event_key_up_scroll_mode_only(Keycode::Down);
                    self.key_hit_up_timer.event_key_up_scroll_mode_only(Keycode::Up);
                }
            },
            Axis::TriggerLeft | Axis::TriggerRight => {
                if value > 100 {
                    self.shoot = true;
                } else {
                    self.shoot = false;
                }
            },
        }
    }

    fn update_game_controller_buttons(&mut self, button: Button, value: bool) {
        match button {
            Button::DPadUp     => self.up = value,
            Button::DPadDown   => self.down = value,
            Button::DPadLeft   => self.left = value,
            Button::DPadRight  => self.right = value,
            Button::A          => self.shoot = value,
            _ => (),
        }
    }

    fn update_key_hit_values_from_game_controller(&mut self, button: Button, value: bool) {
        match button {
            Button::DPadLeft       => self.key_hit_left = value,
            Button::DPadRight      => self.key_hit_right = value,
            Button::A              => self.key_hit_enter = value,
            Button::Back           => self.key_hit_back = value,
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

impl Input for InputManager {
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

    fn event_key_down_scroll_mode_only(&mut self, key: Keycode, current_time: PreciseTime) {
        if self.key != key {
            return;
        }

        match self.state {
            None => {
                self.state = Some(KeyHitState::ScrollMode);
                self.timer.reset(current_time);
                self.key_hit = true;
            },
            _ => {
                if self.timer.check(current_time, 400) {
                    self.key_hit = true;
                }
            },
        }
    }

    fn event_key_up_scroll_mode_only(&mut self, key: Keycode) {
        if self.key != key {
            return;
        }
        self.key_hit = false;
        self.state = None;
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