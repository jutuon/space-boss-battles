/*
src/utils.rs, 2017-08-18

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Miscellaneous utilities.

//use sdl2::TimerSubsystem;
use time::PreciseTime;

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
        let time = self.update_timer.milliseconds(current_time);

        if time >= self.logic_update_time_milliseconds {
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

/// Provides current time as milliseconds for game's components.
pub struct TimeManager {
    //timer_subsystem: TimerSubsystem,
    current_time: TimeMilliseconds,
    start_time: PreciseTime,
}

impl TimeManager {
    /// Create new `TimeManager`.
    pub fn new() -> TimeManager {
        TimeManager {
            //timer_subsystem,
            current_time: TimeMilliseconds(0),
            start_time: PreciseTime::now(),
        }
    }

    /// Get current time.
    pub fn current_time(&self) -> &TimeMilliseconds {
        &self.current_time
    }

    /// Updates `TimeManager`'s current time.
    pub fn update_current_time(&mut self) {
        //self.current_time = TimeMilliseconds(self.timer_subsystem.ticks());
        self.current_time = TimeMilliseconds(self.start_time.to(PreciseTime::now()).num_milliseconds() as u32);
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