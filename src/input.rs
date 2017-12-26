/*
src/input.rs, 2017-09-01

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Input handling.

use cgmath::Point2;

use utils::TimeMilliseconds;

use self::utils::{KeyEvent, KeyHitGenerator};


pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Shoot,
    Select,
    Back,
}

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
    /// Current location of mouse in world coordinates.
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
}

impl InputManager {
    /// Create new InputManger.
    pub fn new() -> InputManager {
        InputManager {
            keyboard: KeyboardManager::new(),
            mouse: MouseManager::new(),
        }
    }

    /// Handle key up event.
    pub fn update_key_up(&mut self, key: Key, current_time: &TimeMilliseconds) {
        self.keyboard.update_keys(key, KeyEvent::KeyUp, current_time);
    }

    /// Handle keyboard key down event.
    pub fn update_key_down(&mut self, key: Key, current_time: &TimeMilliseconds) {
        self.keyboard.update_keys(key, KeyEvent::KeyDown, current_time);
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
    pub fn update(&mut self, current_time: &TimeMilliseconds) {
        self.mouse.reset_button_hits();
        self.keyboard.update(current_time);
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
            mouse_motion: false,
            mouse_button_hit: false,
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
    pub fn update_keys(&mut self, key: Key, key_event: KeyEvent, current_time: &TimeMilliseconds) {
        let (key_down_field, key_hit_field) = match key_event {
            KeyEvent::KeyUp => (false, true),
            KeyEvent::KeyDown => (true, false),
        };

        match key {
            Key::Up => {
                self.up = key_down_field;
                self.key_hit_up.update_from_key_event(key_event, current_time);
            },
            Key::Down => {
                self.down = key_down_field;
                self.key_hit_down.update_from_key_event(key_event, current_time);
            }
            Key::Left => {
                self.left = key_down_field;
                self.key_hit_left.update_from_key_event(key_event, current_time);
            }
            Key::Right => {
                self.right = key_down_field;
                self.key_hit_right.update_from_key_event(key_event, current_time);
            }
            Key::Shoot => self.shoot = key_down_field,
            Key::Select => self.key_hit_enter = key_hit_field,
            Key::Back  => self.key_hit_back = key_hit_field,
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
    pub fn update(&mut self, current_time: &TimeMilliseconds) {
        self.reset_key_hits();

        self.key_hit_up.update(current_time, self.up);
        self.key_hit_down.update(current_time, self.down);
        self.key_hit_left.update(current_time, self.left);
        self.key_hit_right.update(current_time, self.right);
    }
}

// TODO: Touch screen support.

mod utils {

    //! Utilities for `input` module's objects.

    use utils::{Timer, TimeMilliseconds};

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
        milliseconds_between_key_hits: u32,
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

        /// Updates generators state from `KeyEvent`.
        pub fn update_from_key_event(&mut self, key_event: KeyEvent, current_time: &TimeMilliseconds) {
            match key_event {
                KeyEvent::KeyUp => self.up(),
                KeyEvent::KeyDown => self.down(current_time),
            }
        }

        /// Update method will generate key hits if
        ///
        /// * There is enough time passed from the last key hit.
        /// * `key_down` argument is true.
        pub fn update(&mut self, current_time: &TimeMilliseconds, key_down: bool) {
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
        fn down(&mut self, current_time: &TimeMilliseconds) {
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