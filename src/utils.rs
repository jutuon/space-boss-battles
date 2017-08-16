/*
src/utils.rs, 2017-08-16

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Miscellaneous utilities.

use time::PreciseTime;

/// Fps counter.
pub struct FpsCounter {
    frame_count: u32,
    update_time: Timer,
    frame_drop_count: u32,
    fps: u32,
}

impl FpsCounter {
    /// Create new `FpsCounter`.
    pub fn new() -> FpsCounter {
        FpsCounter {
            frame_count: 0,
            update_time: Timer::new(),
            frame_drop_count: 0,
            fps: 0,
        }
    }

    /// Add one frame to frame count.
    pub fn frame(&mut self) {
        self.frame_count += 1;
    }

    /// Add one frame to frame drop count.
    pub fn frame_drop(&mut self) {
        self.frame_drop_count += 1;
    }

    /// Print frame count and frame drop count to standard output.
    ///
    /// Frame drops will only be printed if frame drop count is greater than zero.
    fn print(&self) {
        if self.frame_drop_count == 0 {
            println!("fps: {}", self.frame_count);
        } else {
            println!("fps: {}, frame drops: {}", self.frame_count, self.frame_drop_count);
        }
    }

    /// Update fps count if there is one second from previous update.
    ///
    /// Returns true if the update happened. If argument `print_fps` is true,
    /// print method will be called when fps update happens.
    pub fn update(&mut self, current_time: PreciseTime, print_fps: bool) -> bool {
        if self.update_time.check(current_time, 1000) {
            if print_fps {
                self.print();
            }

            self.fps = self.frame_count;

            self.frame_count = 0;
            self.frame_drop_count = 0;

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

/// Handle timing of logic updates and rendering.
pub struct GameLoopTimer {
    logic_update_time_milliseconds: i64,
    drop_frame: bool,
    update_logic: bool,
    update_timer: Timer,
}

impl GameLoopTimer {
    /// Create new `GameLoopTimer`.
    ///
    /// Argument `logic_update_time_milliseconds` is time between logic updates
    /// in milliseconds.
    pub fn new(logic_update_time_milliseconds: i64) -> GameLoopTimer {
        GameLoopTimer {
            logic_update_time_milliseconds,
            drop_frame: false,
            update_logic: false,
            update_timer: Timer::new(),
        }
    }

    /// Set `update_logic` field true if time between logic updates is equal or more than field's `logic_update_time_milliseconds` value.
    /// If the time between logic updates is more than field's `logic_update_time_milliseconds` value, then drop a frame.
    pub fn update(&mut self, current_time: PreciseTime) {
        self.update_logic = false;
        self.drop_frame = false;

        let time = self.update_timer.milliseconds(current_time);

        if time == self.logic_update_time_milliseconds {
            self.update_logic = true;
            self.drop_frame = false;

            self.update_timer.reset(current_time);
        } else if time > self.logic_update_time_milliseconds {
            self.update_logic = true;
            self.drop_frame = true;

            self.update_timer.reset(current_time);
        }
    }

    /// If this is true, the frame should be dropped.
    pub fn drop_frame(&self) -> bool {
        self.drop_frame
    }

    /// If this is true, the logic should be updated.
    pub fn update_logic(&self) -> bool {
        self.update_logic
    }
}

/// Check time between updates.
pub struct Timer {
    update_time: PreciseTime,
}

impl Timer {
    /// Create new `Timer`.
    ///
    /// This function will call `PreciseTime::now()` to get the current time.
    pub fn new() -> Timer {
        Timer {
            update_time: PreciseTime::now()
        }
    }

    /// Create `Timer` from argument `time`.
    pub fn new_from_time(time: PreciseTime) -> Timer {
        Timer {
            update_time: time
        }
    }

    /// Resets the timer if time between timer and argument `current_time` is equal or greater than
    /// argument `timer_reset_milliseconds`.
    pub fn check(&mut self, current_time: PreciseTime, timer_reset_milliseconds: i64) -> bool {
        if self.milliseconds(current_time) >= timer_reset_milliseconds {
            self.reset(current_time);
            return true;
        }

        false
    }

    /// How much time has elapsed since last timer reset.
    pub fn milliseconds(&self, current_time: PreciseTime) -> i64 {
        self.update_time.to(current_time).num_milliseconds()
    }

    /// Resets timer to time in argument `current_time`.
    pub fn reset(&mut self, current_time: PreciseTime) {
         self.update_time = current_time;
    }
}