/*
src/main.rs, 2017-08-06

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

use sdl2::event::{Event};
use sdl2::keyboard::Keycode;
use sdl2::{GameControllerSubsystem, JoystickSubsystem};

use renderer::{Renderer, OpenGLRenderer};
use logic::Logic;

use input::{InputManager};
use gui::{GUI, GUIEvent};
use gui::settings::{SettingType, SettingEvent};

use time::PreciseTime;

fn main() {
    let sdl_context = sdl2::init().expect("sdl2 init failed");
    let mut event_pump = sdl_context.event_pump().expect("failed to get handle to sdl2 event_pump");

    let video = sdl_context.video().expect("video subsystem init fail");

    let mut renderer = renderer::OpenGLRenderer::new(video);

    let game_controller_subsystem = sdl_context.game_controller().expect("game controller subsystem init failed");
    let joystick_subsystem = sdl_context.joystick().expect("joystick subsystem init failed");
    let mut game = Game::new(game_controller_subsystem, renderer, joystick_subsystem);

    loop {
        if game.quit() {
            break;
        }

        for event in event_pump.poll_iter() {
            game.handle_event(event);
        }

        game.update();

        game.render();
    }

}

pub struct Game {
    game_logic: Logic,
    quit: bool,
    input: InputManager,
    fps_counter: FpsCounter,
    timer: GameLoopTimer,
    gui: GUI,
    renderer: OpenGLRenderer,
}

impl Game {
    pub fn new(controller_subsystem: GameControllerSubsystem, renderer: OpenGLRenderer, joystick_subsystem: JoystickSubsystem) -> Game {
        let game_logic = Logic::new();
        let quit = false;
        let input = InputManager::new(controller_subsystem, joystick_subsystem);
        let fps_counter = FpsCounter::new();
        let timer = GameLoopTimer::new(16);

        let gui = GUI::new();

        Game {game_logic, quit, input, fps_counter, timer, gui, renderer}
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => self.quit = true,
                Event::KeyDown {keycode: Some(key), ..} => self.input.update_key_down(key),
                Event::KeyUp {keycode: Some(key), ..} => self.input.update_key_up(key),
                Event::MouseMotion { x, y, ..} => self.input.update_mouse_motion(self.renderer.screen_coordinates_to_world_coordinates(x, y)),
                Event::MouseButtonUp { x, y, ..} =>  self.input.update_mouse_button_up(self.renderer.screen_coordinates_to_world_coordinates(x, y)),
                Event::ControllerDeviceAdded { which, ..} => self.input.add_game_controller(which as u32),
                Event::ControllerDeviceRemoved { which, ..} => self.input.remove_game_controller(which),
                Event::ControllerAxisMotion { axis, value, ..} => self.input.game_controller_axis_motion(axis, value),
                Event::ControllerButtonDown { button, ..} => self.input.game_controller_button_down(button),
                Event::ControllerButtonUp { button, ..} => self.input.game_controller_button_up(button),
                Event::JoyDeviceAdded { which, ..} => self.input.add_joystick(which as u32),

                Event::JoyAxisMotion { value, axis_idx, .. } => println!("JoyAxisMotion, value: {}, axis_idx: {},", value, axis_idx),
                Event::JoyBallMotion { ball_idx, xrel, yrel, .. } => println!("JoyBallMotion, ball_idx: {}, xrel: {}, yrel: {}", ball_idx, xrel, yrel),
                Event::JoyHatMotion { hat_idx, state, .. } => println!("JoyHatMotion, hat_idx: {}, state as number: {}, state: {:?}", hat_idx, state as u32, state),
                Event::JoyButtonDown { button_idx, .. } => println!("JoyButtonDown, button_idx: {}", button_idx),

                _ => (),
        }
    }

    pub fn render(&mut self) {
        if self.timer.drop_frame() {
            self.fps_counter.frame_drop_count;
            return;
        }

        self.fps_counter.frame();

        self.renderer.start();

        if self.gui.render_game() {
            self.renderer.render(&self.game_logic);
        }

        self.renderer.render_gui(&self.gui);

        self.renderer.end();
    }

    pub fn update(&mut self) {
        let current_time = PreciseTime::now();

        let (fps_updated, fps_count) = self.fps_counter.update(current_time);

        if fps_updated && self.gui.get_gui_fps_counter().show_fps() {
            self.gui.update_fps_counter(fps_count);
        }

        self.timer.update(current_time);

        if self.timer.update_logic() {
            if self.gui.update_game() {
                self.game_logic.update(&self.input);
            }

            match self.gui.handle_event(&mut self.input) {
                None => (),
                Some(GUIEvent::Exit) => self.quit = true,
                Some(GUIEvent::ChangeSetting(SettingType::Boolean(event, value))) => {
                    match event {
                        SettingEvent::FullScreen => self.renderer.full_screen(value),
                        SettingEvent::VSync => self.renderer.v_sync(value),
                        _ => (),
                    };
                }
                _ => (),
            }

            self.input.reset_key_hits();
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

    pub fn update(&mut self, current_time: PreciseTime) -> (bool, u32) {
        if self.update_time.check(current_time, 1000) {
            self.print();
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