/*
src/main.rs, 2017-07-28

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/


extern crate sdl2;
extern crate gl;
extern crate time;
extern crate image;
extern crate cgmath;


mod gui;
mod logic;
mod renderer;
mod input;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use renderer::Renderer;
use logic::Logic;

use input::{InputKeyboard};
use gui::{GUI, GUILayer, GUIEvent};

use time::PreciseTime;

fn main() {
    let sdl_context = sdl2::init().expect("sdl2 init failed");
    let mut event_pump = sdl_context.event_pump().expect("failed to get handle to sdl2 event_pump");

    let video = sdl_context.video().expect("video subsystem init fail");

    let mut renderer = renderer::OpenGLRenderer::new(video);
    let mut game = Game::new();

    loop {
        if game.quit() {
            break;
        }

        for event in event_pump.poll_iter() {
            game.handle_event(event);
        }

        game.update();

        game.render(&mut renderer);
    }

}

pub struct Game {
    game_logic: Logic,
    quit: bool,
    input: InputKeyboard,
    fps_counter: FpsCounter,
    timer: GameLoopTimer,
    gui: GUI,
}

impl Game {
    pub fn new() -> Game {
        let game_logic = Logic::new();
        let quit = false;
        let input = InputKeyboard::new();
        let fps_counter = FpsCounter::new();
        let timer = GameLoopTimer::new(16);

        let gui = GUI::new();

        Game {game_logic, quit, input, fps_counter, timer, gui}
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => self.quit = true,
                Event::KeyDown {keycode: Some(key), ..} => self.input.update_key_down(key),
                Event::KeyUp {keycode: Some(key), ..} => self.input.update_key_up(key),
                _ => (),
        }
    }

    pub fn render<T: Renderer>(&mut self, renderer: &mut T) {
        if self.timer.drop_frame() {
            self.fps_counter.frame_drop_count;
            return;
        }

        self.fps_counter.frame();

        renderer.start();

        if self.gui.render_game() {
            renderer.render(&self.game_logic);
        }

        renderer.render_gui(&self.gui);

        renderer.end();
    }

    pub fn update(&mut self) {
        let current_time = PreciseTime::now();

        self.fps_counter.update(current_time);
        self.timer.update(current_time);

        if self.timer.update_logic() {
            if self.gui.update_game() {
                self.game_logic.update(&self.input);
            }

            match self.gui.handle_event(&mut self.input) {
                None => (),
                Some(GUIEvent::Exit) => self.quit = true,
                _ => (),
            }

            self.input.reset_keyhits();
        }
    }
}

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

    pub fn update(&mut self, current_time: PreciseTime) {
        if self.update_time.check(current_time, 1000) {
            self.print();

            self.fps_count = 0;
            self.frame_drop_count = 0;
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