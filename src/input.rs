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

use time::PreciseTime;

use self::utils::{KeyEvent, KeyHitGenerator};

/// Interface for game components requiring user input information.
///
/// Key hits and button hits will reset to false when method is called.
pub trait Input {
    /// Is up key down currently
    fn up(&self) -> bool;
    /// Is down key down currently
    fn down(&self) -> bool;
    /// Is left key down currently
    fn left(&self) -> bool;
    /// Is right key down currently
    fn right(&self) -> bool;
    /// Is shoot key down currently
    fn shoot(&self) -> bool;

    /// Key hit for up key.
    fn key_hit_up(&mut self) -> bool;
    /// Key hit for down key.
    fn key_hit_down(&mut self) -> bool;
    /// Key hit for left key.
    fn key_hit_left(&mut self) -> bool;
    /// Key hit for right key.
    fn key_hit_right(&mut self) -> bool;
    /// Key hit for enter key.
    fn key_hit_enter(&mut self) -> bool;
    /// Key hit for back key.
    fn key_hit_back(&mut self) -> bool;

    /// Button hit for any mouse button.
    fn mouse_button_hit(&mut self) -> bool;
    /// Is mouse location update occurred.
    /// Resets to false.
    fn mouse_motion(&mut self) -> bool;
    /// Current location of mouse.
    fn mouse_location(&self) -> &Point2<f32>;
}

/// Handles user input events and stores current input state.
///
/// Currently supported input methods are
/// * Keyboard
/// * Mouse
/// * Game controller
pub struct InputManager {
    keyboard: KeyboardManager,
    mouse: MouseManager,
    game_controller: GameControllerManager,
}

impl InputManager {
    /// Create new InputManger.
    pub fn new(game_controller_subsystem: GameControllerSubsystem, joystick_subsystem: JoystickSubsystem) -> InputManager {
        InputManager {
            keyboard: KeyboardManager::new(),
            mouse: MouseManager::new(),
            game_controller: GameControllerManager::new(joystick_subsystem, game_controller_subsystem),
        }
    }

    /// Handle key up event.
    pub fn update_key_up(&mut self, key: Keycode) {
        self.keyboard.update_keys(key, KeyEvent::KeyUp);
    }

    /// Handle keyboard key down event.
    pub fn update_key_down(&mut self, key: Keycode) {
        self.keyboard.update_keys(key, KeyEvent::KeyDown);
    }

    /// Handle mouse motion event.
    pub fn update_mouse_motion(&mut self, point: Point2<f32>) {
        self.mouse.update_mouse_motion(point);
    }

    /// Handle mouse button up event.
    pub fn update_mouse_button_up(&mut self, point: Point2<f32>) {
        self.mouse.update_mouse_button_up(point);
    }

    /// Resets `MouseManager` button hits and updates `KeyboardManager`
    pub fn update(&mut self, current_time: PreciseTime) {
        self.mouse.reset_button_hits();
        self.keyboard.update(current_time);
    }

    /// Handle game controller button up event.
    pub fn game_controller_button_up(&mut self, button: Button) {
        GameControllerManager::handle_button(button, KeyEvent::KeyUp, &mut self.keyboard);
    }

    /// Handle game controller button down event.
    pub fn game_controller_button_down(&mut self, button: Button) {
        GameControllerManager::handle_button(button, KeyEvent::KeyDown, &mut self.keyboard);
    }

    /// Handle game controller axis motion event.
    pub fn game_controller_axis_motion(&mut self, axis: Axis, value: i16) {
        GameControllerManager::handle_axis_motion(axis, value, &mut self.keyboard);
    }

    /// Handle joystick event.
    ///
    /// Adds joystick as `GameController` to `GameControllerManager`.
    /// If there isn't a mapping for joystick, a default mapping will be created and
    /// saved to `Settings`.
    pub fn add_joystick(&mut self, id: u32, settings: &mut Settings) {
        if let Some(mapping) = self.game_controller.add_game_controller_from_joystick_id(id) {
            settings.add_game_controller_mapping(mapping);
        }
    }

    /// Remove game controller from `GameControllerManager`.
    pub fn remove_game_controller(&mut self, id: i32) {
        self.game_controller.remove_game_controller(id);
    }
}

/// Returns the value of boolean reference and sets
/// references value to false.
fn return_and_reset(value: &mut bool) -> bool {
    let original_value: bool = *value;
    *value = false;
    original_value
}

impl Input for InputManager {
    fn up(&self) -> bool    { self.keyboard.up    }
    fn down(&self) -> bool  { self.keyboard.down  }
    fn left(&self) -> bool  { self.keyboard.left  }
    fn right(&self) -> bool { self.keyboard.right }
    fn shoot(&self) -> bool { self.keyboard.shoot }

    fn key_hit_up(&mut self) -> bool     { self.keyboard.key_hit_up.key_hit()    }
    fn key_hit_down(&mut self) -> bool   { self.keyboard.key_hit_down.key_hit()  }
    fn key_hit_left(&mut self) -> bool   { self.keyboard.key_hit_left.key_hit()  }
    fn key_hit_right(&mut self) -> bool  { self.keyboard.key_hit_right.key_hit() }
    fn key_hit_enter(&mut self) -> bool  { return_and_reset(&mut self.keyboard.key_hit_enter) }
    fn key_hit_back(&mut self) -> bool   { return_and_reset(&mut self.keyboard.key_hit_back) }

    fn mouse_button_hit(&mut self) -> bool      { return_and_reset(&mut self.mouse.mouse_button_hit) }
    fn mouse_motion(&mut self) -> bool          { return_and_reset(&mut self.mouse.mouse_motion) }
    fn mouse_location(&self) -> &Point2<f32>    { &self.mouse.mouse_location }
}

/// Store mouse location and button hit
struct MouseManager {
    mouse_motion: bool,
    mouse_button_hit: bool,
    mouse_location: Point2<f32>,
}

impl MouseManager {
    /// Create new `MouseManager`.
    pub fn new() -> MouseManager {
        MouseManager {
            mouse_motion: true,
            mouse_button_hit: true,
            mouse_location: Point2::new(0.0, 0.0),
        }
    }

    /// Reset mouse button hit.
    pub fn reset_button_hits(&mut self) {
        self.mouse_button_hit = false;
    }

    /// Handle mouse motion event.
    pub fn update_mouse_motion(&mut self, point: Point2<f32>) {
        self.mouse_motion = true;
        self.mouse_location = point;
    }

    /// Handle mouse button up event.
    pub fn update_mouse_button_up(&mut self, point: Point2<f32>) {
        self.mouse_button_hit = true;
        self.mouse_location = point;
    }
}

/// Store keyboard state.
///
/// Supports key hits and key down info.
/// For up, down, left and right keys, key hits are implemented with `KeyHitGenerator` which
/// generates key hits if key is kept down.
struct KeyboardManager {
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
}

impl KeyboardManager {

    /// Creates new `KeyboardManager`
    pub fn new() -> KeyboardManager {
        KeyboardManager {
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
        }
    }

    /// Updates `KeyboardManager`'s fields from keyboard event
    pub fn update_keys(&mut self, key: Keycode, key_event: KeyEvent) {
        let (key_down_field, key_hit_field) = match key_event {
            KeyEvent::KeyUp => (false, true),
            KeyEvent::KeyDown => (true, false),
        };

        match key {
            Keycode::Up => {
                self.up = key_down_field;
                self.key_hit_up.update_from_key_event(key_event);
            },
            Keycode::Down => {
                self.down = key_down_field;
                self.key_hit_down.update_from_key_event(key_event);
            }
            Keycode::Left => {
                self.left = key_down_field;
                self.key_hit_left.update_from_key_event(key_event);
            }
            Keycode::Right => {
                self.right = key_down_field;
                self.key_hit_right.update_from_key_event(key_event);
            }
            Keycode::Space      => self.shoot = key_down_field,
            Keycode::Return     => self.key_hit_enter = key_hit_field,
            Keycode::Backspace  => self.key_hit_back = key_hit_field,
            _ => (),
        }
    }

    /// Reset key hit fields and `KeyHitGenerator`s
    fn reset_key_hits(&mut self) {
        self.key_hit_enter = false;
        self.key_hit_back = false;

        self.key_hit_up.clear();
        self.key_hit_down.clear();
        self.key_hit_left.clear();
        self.key_hit_right.clear();
    }

    /// Reset key hit fields and `KeyHitGenerator`s and updates `KeyHitGenerator`s
    pub fn update(&mut self, current_time: PreciseTime) {
        self.reset_key_hits();

        self.key_hit_up.update(current_time, self.up);
        self.key_hit_down.update(current_time, self.down);
        self.key_hit_left.update(current_time, self.left);
        self.key_hit_right.update(current_time, self.right);
    }
}

type GameControllerMapping = String;

/// Add and remove game controllers, route game controller events to `KeyboardManager`
struct GameControllerManager {
    joystick_subsystem: JoystickSubsystem,
    game_controller_subsystem: GameControllerSubsystem,
    game_controllers: Vec<GameController>,
}

impl GameControllerManager {
    /// Create new `GameControllerManager`
    fn new(joystick_subsystem: JoystickSubsystem, game_controller_subsystem: GameControllerSubsystem) -> GameControllerManager {
        GameControllerManager {
            joystick_subsystem,
            game_controller_subsystem,
            game_controllers: Vec::new(),
        }
    }

    /// Adds new game controller from SDL2 joystick id to `GameControllerManager`.
    ///
    /// If the joystick doesn't have a game controller mapping, method will create default
    /// mapping for the joystick and return the created mapping.
    ///
    /// If there is an error it will be printed to standard output.
    pub fn add_game_controller_from_joystick_id(&mut self, id: u32) -> Option<GameControllerMapping> {
        let game_controller_mapping = if !self.game_controller_subsystem.is_game_controller(id) {
            let joystick_name;
            match self.joystick_subsystem.name_for_index(id) {
                Ok(name) => joystick_name = name,
                Err(error) => {
                    println!("error: {}", error);
                    return None;
                }
            }

            let mut joystick_guid;
            match self.joystick_subsystem.device_guid(id) {
                Ok(guid) => joystick_guid = guid.to_string(),
                Err(error) => {
                    println!("error: {}", error);
                    return None;
                }
            }

            // https://wiki.libsdl.org/SDL_GameControllerAddMapping
            joystick_guid.push(',');
            joystick_guid.push_str(&joystick_name);
            joystick_guid.push_str(", a:b2, b:b1, y:b0, x:b3, start:b9, guide:b12, back:b8, dpup:h0.1, dpleft:h0.8, dpdown:h0.4, dpright:h0.2, leftshoulder:b6, rightshoulder:b7, leftstick:b10, rightstick:b11, leftx:a0, lefty:a1, rightx:a3, righty:a2, lefttrigger:b4, righttrigger:b5");

            match self.game_controller_subsystem.add_mapping(&joystick_guid) {
                Ok(_) => {
                    println!("default game controller mapping loaded for joystick with id {}", id);
                    Some(joystick_guid)
                },
                Err(error) => {
                    println!("error: {}", error);
                    return None
                }
            }
        } else {
            None
        };

        match self.game_controller_subsystem.open(id) {
            Ok(controller) => {
                self.game_controllers.push(controller);
                println!("game controller with id {} added", id);
            },
            Err(integer_or_sdl_error) => println!("game controller error: {}", integer_or_sdl_error),
        }

        game_controller_mapping
    }

    /// Remove game controller which has same id as argument `id`.
    ///
    /// Game controller will be removed from `GameControllerManager`'s `Vec<GameController>`
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
            println!("game controller with id {} removed", id);
        }
    }


    /// Forwards game controller's axis event to `KeyboardManager`.
    pub fn handle_axis_motion(axis: Axis, value: i16, keyboard: &mut KeyboardManager) {
        match axis {
            Axis::LeftX | Axis::RightX => {
                if value > 10000 {
                    keyboard.update_keys(Keycode::Right, KeyEvent::KeyDown);
                } else if value < -10000 {
                    keyboard.update_keys(Keycode::Left, KeyEvent::KeyDown);
                } else {
                    if keyboard.left {
                        keyboard.update_keys(Keycode::Left, KeyEvent::KeyUp);
                    }
                    if keyboard.right {
                        keyboard.update_keys(Keycode::Right, KeyEvent::KeyUp);
                    }
                }
            },
            Axis::LeftY | Axis::RightY => {
                if value > 10000 {
                    keyboard.update_keys(Keycode::Down, KeyEvent::KeyDown);
                } else if value < -10000 {
                    keyboard.update_keys(Keycode::Up, KeyEvent::KeyDown);
                } else {
                    if keyboard.down {
                        keyboard.update_keys(Keycode::Down, KeyEvent::KeyUp);
                    }
                    if keyboard.up {
                        keyboard.update_keys(Keycode::Up, KeyEvent::KeyUp);
                    }
                }
            },
            Axis::TriggerLeft | Axis::TriggerRight => {
                if value > 100 {
                    keyboard.shoot = true;
                } else {
                    keyboard.shoot = false;
                }
            },
        }
    }

    /// Forwards game controller's button event to `KeyboardManager`.
    pub fn handle_button(button: Button, key_event: KeyEvent, keyboard: &mut KeyboardManager) {
        match button {
            Button::DPadUp     => keyboard.update_keys(Keycode::Up, key_event),
            Button::DPadDown   => keyboard.update_keys(Keycode::Down, key_event),
            Button::DPadLeft   => keyboard.update_keys(Keycode::Left, key_event),
            Button::DPadRight  => keyboard.update_keys(Keycode::Right, key_event),
            Button::A          => {
                keyboard.update_keys(Keycode::Space, key_event.clone());
                keyboard.update_keys(Keycode::Return, key_event);
            },
            Button::Back       => keyboard.update_keys(Keycode::Backspace, key_event),
            _ => (),
        }
    }
}

// TODO: Touch screen support.

mod utils {

    //! Utilities for `input` module's objects.

    use time::PreciseTime;
    use Timer;

    /// Key press states.
    #[derive(Clone)]
    pub enum KeyEvent {
        KeyUp,
        KeyDown,
    }

    /// KeyHitGenerator's states.
    enum KeyHitState {
        /// Normal key hits.
        NormalMode,
        /// Generator generated key hits.
        ScrollMode,
    }

    /// Generate key hits.
    ///
    /// Generates key hits from key up event and if the key is pressed down
    /// long enough, the generator will generate multiple key hits.
    pub struct KeyHitGenerator {
        milliseconds_between_key_hits: i64,
        timer: Timer,
        state: Option<KeyHitState>,
        key_hit: bool,
    }

    impl KeyHitGenerator {
        /// Create new `KeyHitGenerator` which `milliseconds_between_key_hits` field is set to `300`.
        pub fn new() -> KeyHitGenerator {
            KeyHitGenerator {
                milliseconds_between_key_hits: 300,
                timer: Timer::new(),
                state: None,
                key_hit: false,
            }
        }

        /// Set time between key hits in milliseconds.
        pub fn set_milliseconds_between_key_hits(mut self, milliseconds: i64) -> KeyHitGenerator {
            let milliseconds = if milliseconds <= 0 {
                1
            } else {
                milliseconds
            };

            self.milliseconds_between_key_hits = milliseconds;

            self
        }


        /// Updates generators state from `KeyEvent`.
        pub fn update_from_key_event(&mut self, key_event: KeyEvent) {
            match key_event {
                KeyEvent::KeyUp => self.up(),
                KeyEvent::KeyDown => self.down(),
            }
        }

        /// Update method will generate key hits if
        ///
        /// * There is enough time passed from the last key hit.
        /// * `key_down` argument is true.
        pub fn update(&mut self, current_time: PreciseTime, key_down: bool) {
            if !key_down {
                return;
            }

            match self.state {
                Some(KeyHitState::NormalMode) => {
                    if self.timer.check(current_time, self.milliseconds_between_key_hits) {
                        self.state = Some(KeyHitState::ScrollMode);
                        self.key_hit = true;
                    }
                },
                Some(KeyHitState::ScrollMode) => {
                    if self.timer.check(current_time, self.milliseconds_between_key_hits) {
                        self.key_hit = true;
                    }
                },
                _ => (),
            }
        }

        /// Handle key down event.
        ///
        /// Sets generators state to `Some(KeyHitState::NormalMode)` and resets
        /// generators internal timer if generators current state is `None`.
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

        /// Handle key up event.
        ///
        /// Creates a key hit if method is called when
        /// generators state is `Some(KeyHitState::NormalMode)`.
        ///
        /// Generators state will be set to `None`.
        fn up(&mut self) {
            if let Some(KeyHitState::NormalMode) = self.state {
                self.key_hit = true;
            } else {
                self.key_hit = false;
            };

            self.state = None;
        }

        /// Returns true if key hit has been happened.
        ///
        /// This method will also clear the current key hit.
        pub fn key_hit(&mut self) -> bool {
            if self.key_hit {
                self.clear();
                true
            } else {
                false
            }
        }

        /// Clears current key hit.
        pub fn clear(&mut self) {
            self.key_hit = false;
        }
    }
}