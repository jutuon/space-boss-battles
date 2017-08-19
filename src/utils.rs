/*
src/utils.rs, 2017-08-19

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Miscellaneous utilities.

//use sdl2::TimerSubsystem;
use std::time::Instant;
use LOGIC_TARGET_FPS;
use LOGIC_MAX_FPS;
const TARGET_FRAME_TIME_MICROSECONDS: f32 = 1_000_000.0 / LOGIC_TARGET_FPS as f32;

const DELTA_TIME_AT_LOGIC_MAX_FPS: f32 = LOGIC_TARGET_FPS as f32 / LOGIC_MAX_FPS as f32;

const LOGIC_MAX_UPDATES_MICROSECONDS: u32 = 1_000_000/LOGIC_MAX_FPS;

/// Fps counter.
pub struct FpsCounter {
    frame_count: u32,
    update_time: Timer,
    fps: u32,
}

impl FpsCounter {
    /// Create new `FpsCounter`.
    pub fn new() -> FpsCounter {
        FpsCounter {
            frame_count: 0,
            update_time: Timer::new(),
            fps: 0,
        }
    }

    /// Add one frame to frame count.
    pub fn frame(&mut self) {
        self.frame_count += 1;
    }

    /// Print fps to standard output.
    fn print(&self) {
        println!("fps: {}", self.fps);
    }

    /// Update fps count if there is one second from previous update.
    ///
    /// Returns true if the update happened. If argument `print_fps` is true,
    /// print method will be called when fps update happens.
    pub fn update(&mut self, current_time: &TimeMilliseconds, print_fps: bool) -> bool {
        if self.update_time.check(current_time, 1000) {
            self.fps = self.frame_count;

            if print_fps {
                self.print();
            }

            self.frame_count = 0;

            true
        } else {
            false
        }
    }

    /// Get current fps value
    pub fn fps(&self) -> u32 {
        self.fps
    }

}

/// Handle timing of logic updates.
pub struct GameLoopTimer {
    logic_update_time_milliseconds: u32,
    update_logic: bool,
    update_timer: Timer,
}

impl GameLoopTimer {
    /// Create new `GameLoopTimer`.
    ///
    /// Argument `logic_update_time_milliseconds` is time between logic updates
    /// in milliseconds.
    pub fn new(logic_update_time_milliseconds: u32) -> GameLoopTimer {
        GameLoopTimer {
            logic_update_time_milliseconds,
            update_logic: false,
            update_timer: Timer::new(),
        }
    }

    /// Set `update_logic` field true if time between logic updates is equal or more than field's `logic_update_time_milliseconds` value.
    pub fn update(&mut self, current_time: &TimeMilliseconds) {
        if self.update_timer.check(current_time, self.logic_update_time_milliseconds) {
            self.update_logic = true;
            self.update_timer.reset(current_time);
        } else {
            self.update_logic = false;
        }
    }

    /// If this is true, the logic should be updated.
    pub fn update_logic(&self) -> bool {
        self.update_logic
    }
}

/// Time handling for game logic.
///
/// Provides delta time for moving objects at constant speed if FPS value is low and
/// game logic specific global time, so pausing the game will not have effect on game logic.
pub struct GameTimeManager {
    current_game_time: TimeMilliseconds,
    previous_game_time: TimeMilliseconds,
    logic_update_start: Option<Instant>,
    delta_time: f32,
    previous_frame_update: Instant,
}

impl GameTimeManager {
    /// Creates new `GameTimeManager`.
    fn new() -> GameTimeManager {
        GameTimeManager {
            current_game_time: TimeMilliseconds(0),
            previous_game_time: TimeMilliseconds(0),
            logic_update_start: None,
            delta_time: 1.0,
            previous_frame_update: Instant::now(),
        }
    }

    /// Get current game time.
    pub fn time(&self) -> &TimeMilliseconds {
        &self.current_game_time
    }

    // FIXME: current_game_time will overflow after some days.

    /// Updates delta time and game time.
    fn update(&mut self, current_time: Instant, game_logic_running: bool) {
        // Game time calculations.
        if game_logic_running {
            if let Some(logic_start) = self.logic_update_start {
                let time = current_time.duration_since(logic_start);
                self.current_game_time = TimeMilliseconds(self.previous_game_time.0 + time.subsec_nanos() / 1_000_000 + (time.as_secs() as u32)*1000);
            } else {
                self.logic_update_start = Some(current_time);
            }
        } else {
            if let Some(_) = self.logic_update_start {
                self.previous_game_time = self.current_game_time.clone();
                self.logic_update_start = None;
            }
        }

        // Delta time calculations.
        let microseconds_between_frames = current_time.duration_since(self.previous_frame_update).subsec_nanos() / 1000;

        if microseconds_between_frames < LOGIC_MAX_UPDATES_MICROSECONDS {
            self.delta_time = DELTA_TIME_AT_LOGIC_MAX_FPS;
        } else {
            self.delta_time = microseconds_between_frames as f32 / TARGET_FRAME_TIME_MICROSECONDS;
        }

        self.previous_frame_update = current_time;
    }

    /// Difference between real frame time and target frame time. Value should be between [1.0, f32::MAX].
    ///
    /// Multiply all movement values in logic code with this, so objects will move at same speed when FPS is low.
    ///
    /// Note that delta time will not work if updates takes one second or more time.
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }
}

// FIXME: current_time will overflow after some days.

/// Provides current time for game's components.
pub struct TimeManager {
    //timer_subsystem: TimerSubsystem,
    current_time: TimeMilliseconds,
    start_time: Instant,
    game_time: GameTimeManager,
}

impl TimeManager {
    /// Create new `TimeManager`.
    pub fn new() -> TimeManager {
        TimeManager {
            //timer_subsystem,
            current_time: TimeMilliseconds(0),
            start_time: Instant::now(),
            game_time: GameTimeManager::new(),
        }
    }

    /// Get current time.
    pub fn current_time(&self) -> &TimeMilliseconds {
        &self.current_time
    }

    /// Get game time manager.
    pub fn game_time_manager(&self) -> &GameTimeManager {
        &self.game_time
    }

    /// Updates `TimeManager`'s current time and `GameTimeManager`'s time and delta time.
    pub fn update_time(&mut self, game_logic_running: bool) {
        //self.current_time = TimeMilliseconds(self.timer_subsystem.ticks());

        let current_instant = Instant::now();

        let time = current_instant.duration_since(self.start_time);
        self.current_time = TimeMilliseconds(time.subsec_nanos() / 1_000_000 + (time.as_secs() as u32)*1000);

        self.game_time.update(current_instant, game_logic_running);
    }
}

/// Wrapper type for time as milliseconds.
pub struct TimeMilliseconds(u32);

impl TimeMilliseconds {
    /// Private version of `Clone` trait's clone method.
    fn clone(&self) -> TimeMilliseconds {
        TimeMilliseconds(self.0)
    }
}

/// Check time between updates.
pub struct Timer {
    update_time: TimeMilliseconds,
}

impl Timer {
    /// Create new `Timer` initialized to zero milliseconds.
    pub fn new() -> Timer {
        Timer {
            update_time: TimeMilliseconds(0),
        }
    }

    /// Create `Timer` from argument `time`.
    pub fn new_from_time(time: &TimeMilliseconds) -> Timer {
        Timer {
            update_time: time.clone(),
        }
    }

    /// Resets the timer if time between timer and argument `current_time` is equal or greater than
    /// argument `timer_reset_milliseconds`.
    pub fn check(&mut self, current_time: &TimeMilliseconds, timer_reset_milliseconds: u32) -> bool {
        if self.milliseconds(current_time) >= timer_reset_milliseconds {
            self.reset(current_time);
            return true;
        }

        false
    }

    /// How much time has elapsed since last timer reset.
    pub fn milliseconds(&self, current_time: &TimeMilliseconds) -> u32 {
        // Current time should always be equal or greater than self.update_time.0
        // so there won't be underflow from subtraction.
        current_time.0 - self.update_time.0
    }

    /// Resets timer to time in argument `current_time`.
    pub fn reset(&mut self, current_time: &TimeMilliseconds) {
         self.update_time = current_time.clone();
    }
}