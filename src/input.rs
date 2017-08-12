/*
src/input.rs, 2017-08-12

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

use settings::Settings;

use Timer;
use time::PreciseTime;

#[derive(Clone)]
enum KeyEvent {
    KeyUp,
    KeyDown,
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

pub struct InputManager {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    shoot: bool,

    key_hit_left: KeyHitGenerator,
    key_hit_right: KeyHitGenerator,
    key_hit_up: KeyHitGenerator,
    key_hit_down: KeyHitGenerator,

    key_hit_enter: bool,
    key_hit_back: bool,

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

            key_hit_left: KeyHitGenerator::new(),
            key_hit_right: KeyHitGenerator::new(),
            key_hit_up: KeyHitGenerator::new(),
            key_hit_down: KeyHitGenerator::new(),

            key_hit_enter: false,
            key_hit_back: false,

            mouse_motion: true,
            mouse_button_hit: true,
            mouse_location: Point2::new(0.0, 0.0),

            game_controller_subsystem: game_controller_subsystem,
            game_controllers: Vec::new(),

            joystick_subsystem: joystick_subsystem,
        }
    }

    fn reset_key_hits(&mut self) {
        self.key_hit_enter = false;
        self.key_hit_back = false;

        self.key_hit_up.reset();
        self.key_hit_down.reset();
        self.key_hit_left.reset();
        self.key_hit_right.reset();

        self.mouse_button_hit = false;
    }

    pub fn update_key_up(&mut self, key: Keycode) {
        self.update_keys(key, KeyEvent::KeyUp);
    }

    pub fn update_key_down(&mut self, key: Keycode) {
        self.update_keys(key, KeyEvent::KeyDown);
    }

    fn update_keys(&mut self, key: Keycode, key_event: KeyEvent) {
        let (key_down_field, key_hit_field) = match key_event {
            KeyEvent::KeyUp => (false, true),
            KeyEvent::KeyDown => (true, false),
        };

        match key {
            Keycode::Up     => {
                self.up = key_down_field;
                self.key_hit_up.update_from_key_event(key_event);
            },
            Keycode::Down   => {
                self.down = key_down_field;
                self.key_hit_down.update_from_key_event(key_event);
            }
            Keycode::Left   => {
                self.left = key_down_field;
                self.key_hit_left.update_from_key_event(key_event);
            }
            Keycode::Right  => {
                self.right = key_down_field;
                self.key_hit_right.update_from_key_event(key_event);
            }
            Keycode::Space  => self.shoot = key_down_field,
            Keycode::Return     => self.key_hit_enter = key_hit_field,
            Keycode::Backspace  => self.key_hit_back = key_hit_field,
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

    pub fn update(&mut self, current_time: PreciseTime) {
        self.reset_key_hits();

        self.key_hit_up.update(current_time, self.up);
        self.key_hit_down.update(current_time, self.down);
        self.key_hit_left.update(current_time, self.left);
        self.key_hit_right.update(current_time, self.right);
    }

    fn add_game_controller(&mut self, id: u32) {
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

    pub fn add_joystick(&mut self, id: u32, settings: &mut Settings) {
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
                    settings.add_game_controller_mapping(joystick_guid);
                    self.add_game_controller(id);
                },
                Err(error) => println!("error: {}", error),
            }
        } else {
            self.add_game_controller(id);
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
        self.handle_game_controller_button(button, KeyEvent::KeyUp);
    }

    pub fn game_controller_button_down(&mut self, button: Button) {
        self.handle_game_controller_button(button, KeyEvent::KeyDown);
    }

    pub fn game_controller_axis_motion(&mut self, axis: Axis, value: i16) {
        match axis {
            Axis::LeftX | Axis::RightX => {
                if value > 10000 {
                    self.update_key_down(Keycode::Right);
                } else if value < -10000 {
                    self.update_key_down(Keycode::Left);
                } else {
                    if self.left {
                        self.update_key_up(Keycode::Left);
                    }

                    if self.right {
                        self.update_key_up(Keycode::Right);
                    }
                }
            },
            Axis::LeftY | Axis::RightY => {
                if value > 10000 {
                    self.update_key_down(Keycode::Down);
                } else if value < -10000 {
                    self.update_key_down(Keycode::Up);
                } else {
                    if self.down {
                        self.update_key_up(Keycode::Down);
                    }
                    if self.up {
                        self.update_key_up(Keycode::Up);
                    }
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

    fn handle_game_controller_button(&mut self, button: Button, key_event: KeyEvent) {
        match button {
            Button::DPadUp     => self.update_keys(Keycode::Up, key_event),
            Button::DPadDown   => self.update_keys(Keycode::Down, key_event),
            Button::DPadLeft   => self.update_keys(Keycode::Left, key_event),
            Button::DPadRight  => self.update_keys(Keycode::Right, key_event),
            Button::A          => {
                self.update_keys(Keycode::Space, key_event.clone());
                self.update_keys(Keycode::Return, key_event);
            },
            Button::Back       => self.update_keys(Keycode::Backspace, key_event),
            _ => (),
        }
    }
}

fn return_and_reset(value: &mut bool) -> bool {
    let original_value: bool = *value;
    *value = false;
    original_value
}

impl Input for InputManager {
    fn up(&self) -> bool    { self.up    }
    fn down(&self) -> bool  { self.down  }
    fn left(&self) -> bool  { self.left  }
    fn right(&self) -> bool { self.right }
    fn shoot(&self) -> bool { self.shoot }

    fn key_hit_up(&mut self) -> bool     { self.key_hit_up.return_and_reset()    }
    fn key_hit_down(&mut self) -> bool   { self.key_hit_down.return_and_reset()  }
    fn key_hit_left(&mut self) -> bool   { self.key_hit_left.return_and_reset()  }
    fn key_hit_right(&mut self) -> bool  { self.key_hit_right.return_and_reset() }
    fn key_hit_enter(&mut self) -> bool  { return_and_reset(&mut self.key_hit_enter) }
    fn key_hit_back(&mut self) -> bool   { return_and_reset(&mut self.key_hit_back) }

    fn mouse_button_hit(&mut self) -> bool      { return_and_reset(&mut self.mouse_button_hit) }
    fn mouse_motion(&mut self) -> bool          { return_and_reset(&mut self.mouse_motion) }
    fn mouse_location(&self) -> &Point2<f32>    { &self.mouse_location }
}


/// KeyHitGenerator's state enum
enum KeyHitState {
    NormalMode,
    ScrollMode,
}

/// Generate key hits
///
/// Generates key hits from key up event and if the key is pressed down
/// long enough
struct KeyHitGenerator {
    timer: Timer,
    state: Option<KeyHitState>,
    key_hit: bool,
}

impl KeyHitGenerator {
    fn new() -> KeyHitGenerator {
        KeyHitGenerator {
            timer: Timer::new(),
            state: None,
            key_hit: false,
        }
    }

    pub fn update_from_key_event(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent::KeyUp => self.up(),
            KeyEvent::KeyDown => self.down(),
        }
    }

    pub fn update(&mut self, current_time: PreciseTime, key_down: bool) {
        if !key_down {
            return;
        }

        match self.state {
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
            },
            _ => (),
        }
    }

    fn down(&mut self) {
        let current_time = PreciseTime::now();

        match self.state {
            None => {
                self.state = Some(KeyHitState::NormalMode);
                self.timer.reset(current_time);
            },
            _ => (),
        }
    }

    fn up(&mut self) {
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