/*
src/main.rs, 2017-08-12

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
extern crate rand;


pub mod gui;
pub mod logic;
pub mod renderer;
pub mod input;
pub mod settings;
pub mod audio;

use sdl2::event::{Event};
use sdl2::keyboard::Keycode;
use sdl2::{GameControllerSubsystem, JoystickSubsystem, AudioSubsystem};

use renderer::{Renderer, OpenGLRenderer};
use logic::Logic;

use input::{InputManager};
use gui::{GUI, GUIEvent};
use gui::components::GUIUpdatePosition;

use settings::{Settings};

use time::PreciseTime;

use std::env;

use audio::AudioManager;

pub const COMMAND_LINE_HELP_TEXT: &str = "
Space Boss Battles command line options:
--help|-h         - show this text
--fps             - print fps to standard output
--joystick-events - print joystick events to standard output
";

fn main() {
    if check_help_option() {
        println!("{}", COMMAND_LINE_HELP_TEXT);
        return;
    }

    let sdl_context = sdl2::init().expect("sdl2 init failed");
    println!("SDL2 version: {}", sdl2::version::version());
    let audio_subsystem = sdl_context.audio().expect("error");

    let mut event_pump = sdl_context.event_pump().expect("failed to get handle to sdl2 event_pump");

    let video = sdl_context.video().expect("video subsystem init fail");

    let renderer = renderer::OpenGLRenderer::new(video);

    let game_controller_subsystem = sdl_context.game_controller().expect("game controller subsystem init failed");
    let joystick_subsystem = sdl_context.joystick().expect("joystick subsystem init failed");
    let mut game = Game::new(game_controller_subsystem, renderer, joystick_subsystem, audio_subsystem);


    for event in event_pump.poll_iter() {
        match event {
            Event::Quit{..} | Event::JoyDeviceAdded{..} => game.handle_event(event),
            _ => (),
        }
    }

    loop {
        if game.quit() {
            game.save_settings();
            break;
        }

        for event in event_pump.poll_iter() {
            game.handle_event(event);
        }

        game.update();

        game.render();
    }

}

fn check_help_option() -> bool {
    let args = env::args();

    for arg in args.skip(1) {
        if arg == "--help" || arg == "-h" {
            return true;
        }
    }

    false
}

pub struct Game {
    game_logic: Logic,
    quit: bool,
    input: InputManager,
    fps_counter: FpsCounter,
    timer: GameLoopTimer,
    gui: GUI,
    renderer: OpenGLRenderer,
    settings: Settings,
    audio_manager: AudioManager,
}

impl Game {
    pub fn new(mut controller_subsystem: GameControllerSubsystem, mut renderer: OpenGLRenderer, joystick_subsystem: JoystickSubsystem, audio_subsystem: AudioSubsystem) -> Game {
        let mut game_logic = Logic::new();
        let quit = false;

        let settings = Settings::new(&mut controller_subsystem);

        let input = InputManager::new(controller_subsystem, joystick_subsystem);
        let fps_counter = FpsCounter::new();
        let timer = GameLoopTimer::new(16);

        let mut gui = GUI::new(&settings);
        gui.update_position_from_half_screen_width(renderer.half_screen_width_world_coordinates());

        let mut audio_manager = AudioManager::new(audio_subsystem);
        settings.apply_current_settings(&mut renderer, &mut gui, &mut game_logic, &mut audio_manager);


        audio_manager.play_music();

        Game { game_logic, quit, input, fps_counter, timer, gui, renderer, settings, audio_manager }
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
                Event::ControllerDeviceRemoved { which, ..} => self.input.remove_game_controller(which),
                Event::ControllerAxisMotion { axis, value, ..} => self.input.game_controller_axis_motion(axis, value),
                Event::ControllerButtonDown { button, ..} => self.input.game_controller_button_down(button),
                Event::ControllerButtonUp { button, ..} => self.input.game_controller_button_up(button),
                Event::JoyDeviceAdded { which, ..} => self.input.add_joystick(which as u32, &mut self.settings),
                _ => (),
        }

        if self.settings.print_joystick_events() {
            match event {
                Event::JoyAxisMotion { value, axis_idx, .. } => println!("JoyAxisMotion, value: {}, axis_idx: {},", value, axis_idx),
                Event::JoyBallMotion { ball_idx, xrel, yrel, .. } => println!("JoyBallMotion, ball_idx: {}, xrel: {}, yrel: {}", ball_idx, xrel, yrel),
                Event::JoyHatMotion { hat_idx, state, .. } => println!("JoyHatMotion, hat_idx: {}, state as number: {}, state: {:?}", hat_idx, state as u32, state),
                Event::JoyButtonDown { button_idx, .. } => println!("JoyButtonDown, button_idx: {}", button_idx),
                _ => (),
            }
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

        let (fps_updated, fps_count) = self.fps_counter.update(current_time, self.settings.print_fps_count());

        if fps_updated && self.gui.get_gui_fps_counter().show_fps() {
            self.gui.update_fps_counter(fps_count);
        }

        self.timer.update(current_time);

        if self.timer.update_logic() {
            if self.gui.update_game() {
                self.game_logic.update(&self.input, &mut self.gui, self.audio_manager.sound_effect_manager_mut());
            }

            match self.gui.handle_input(&mut self.input) {
                None => (),
                Some(GUIEvent::Exit) => self.quit = true,
                Some(GUIEvent::ChangeSetting(new_setting_value)) => {
                    self.settings.update_setting(new_setting_value);
                    Settings::apply_setting(new_setting_value, &mut self.renderer, &mut self.gui, &mut self.game_logic, &mut self.audio_manager);
                },
                Some(GUIEvent::NewGame(difficulty)) => self.game_logic.reset_game(&mut self.gui, difficulty, 0),
                Some(GUIEvent::NextLevel) => self.game_logic.reset_to_next_level(&mut self.gui),
                _ => (),
            }

            self.input.update(current_time);
        }
    }

    pub fn save_settings(&self) {
        self.settings.save();
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