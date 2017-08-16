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

pub struct FpsCounter {
    fps_count: u32,
    update_time: Timer,
    frame_drop_count: u32,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {fps_count: 0, update_time: Timer::new(), frame_drop_count: 0}
    }

    pub fn frame(&mut self) {
        self.fps_count += 1;
    }

    pub fn frame_drop(&mut self) {
        self.frame_drop_count += 1;
    }

    fn print(&self) {
        if self.frame_drop_count == 0 {
            println!("fps: {}", self.fps_count);
        } else {
            println!("fps: {}, frame drops: {}", self.fps_count, self.frame_drop_count);
        }
    }

    pub fn update(&mut self, current_time: PreciseTime, print_fps: bool) -> (bool, u32) {
        if self.update_time.check(current_time, 1000) {
            if print_fps {
                self.print();
            }

            let return_value = (true, self.fps_count);

            self.fps_count = 0;
            self.frame_drop_count = 0;

            return_value
        } else {
            (false, 0)
        }
    }

}

pub struct GameLoopTimer {
    logic_update_time_milliseconds: i64,
    drop_frame: bool,
    update_logic: bool,
    update_timer: Timer,
}

impl GameLoopTimer {
    pub fn new(logic_update_time_milliseconds: i64) -> GameLoopTimer {
        let drop_frame = false;
        let update_logic = false;
        let update_timer = Timer::new();

        GameLoopTimer {logic_update_time_milliseconds, drop_frame, update_logic, update_timer}
    }

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

    pub fn drop_frame(&self) -> bool {
        self.drop_frame
    }

    pub fn update_logic(&self) -> bool {
        self.update_logic
    }
}

pub struct Timer {
    update_time: PreciseTime,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {update_time: PreciseTime::now()}
    }

    pub fn new_from_time(time: PreciseTime) -> Timer {
        Timer { update_time: time }
    }

    pub fn check(&mut self, current_time: PreciseTime, timer_reset_milliseconds: i64) -> bool {
        if self.milliseconds(current_time) >= timer_reset_milliseconds {
            self.reset(current_time);
            return true;
        }

        false
    }

    pub fn milliseconds(&self, current_time: PreciseTime) -> i64 {
        self.update_time.to(current_time).num_milliseconds()
    }

    pub fn reset(&mut self, current_time: PreciseTime) {
         self.update_time = current_time;
    }
}