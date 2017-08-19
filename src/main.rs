/*
src/main.rs, 2017-08-19

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/


extern crate sdl2;
extern crate gl;
extern crate image;
extern crate cgmath;
extern crate rand;


pub mod gui;
pub mod logic;
pub mod renderer;
pub mod input;
pub mod settings;
pub mod audio;
pub mod utils;

use sdl2::event::{Event};
use sdl2::keyboard::Keycode;
use sdl2::{GameControllerSubsystem, JoystickSubsystem};

use renderer::{Renderer, OpenGLRenderer};
use logic::Logic;

use input::{InputManager};
use gui::{GUI, GUIEvent, GUIState};

use settings::{Settings, Arguments};

use std::env;

use audio::{AudioManager, SoundEffectPlayer};

use utils::{FpsCounter, GameLoopTimer, TimeManager};

/// Base value for `GameTimeManager`'s delta time.
pub const LOGIC_TARGET_FPS: u32 = 60;

/// Max value for logic updates per seconds. This limit exist to avoid
/// possible floating point errors from extremely small delta time values
/// from `GameTimeManager`.
///
/// Current max value for this is 1000, because GameLoopTimer only handles milliseconds.
pub const LOGIC_MAX_FPS: u32 = 1000;

const LOGIC_MAX_UPDATES_MILLISECONDS: u32 = 1000/LOGIC_MAX_FPS;

pub const COMMAND_LINE_HELP_TEXT: &str = "
Space Boss Battles command line options:
--help|-h         - show this text
--fps             - print fps to standard output
--joystick-events - print joystick events to standard output
--music FILE_PATH - set path to music file
";

fn main() {
    let arguments = match Arguments::parse(env::args()) {
        Ok(arguments) => arguments,
        Err(unknown_argument) => {
            println!("unknown argument: \"{}\"", unknown_argument);
            println!("{}", COMMAND_LINE_HELP_TEXT);
            return;
        }
    };

    if arguments.show_help() {
        println!("{}", COMMAND_LINE_HELP_TEXT);
        return;
    }

    let sdl_context = sdl2::init().expect("sdl2 init failed");
    println!("SDL2 version: {}", sdl2::version::version());

/*
    let audio_subsystem = sdl_context.audio().expect("error");

    println!("SDL2 current audio driver: {}", audio_subsystem.current_audio_driver());

    if let Some(number) = audio_subsystem.num_audio_playback_devices() {
        for i in 0..number {
            println!("playback device index: {}, name: {}", i, audio_subsystem.audio_playback_device_name(i).expect("error"));
        }
    }
*/

    let mut event_pump = sdl_context.event_pump().expect("failed to get handle to sdl2 event_pump");

    let video = sdl_context.video().expect("video subsystem init fail");

    let renderer = renderer::OpenGLRenderer::new(video);

    let game_controller_subsystem = sdl_context.game_controller().expect("game controller subsystem init failed");
    let joystick_subsystem = sdl_context.joystick().expect("joystick subsystem init failed");

    let mut game = Game::new(game_controller_subsystem, renderer, joystick_subsystem, arguments);


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
    update_game: bool,
    render_game: bool,
    time_manager: TimeManager,
}

impl Game {
    pub fn new(
                mut controller_subsystem: GameControllerSubsystem,
                mut renderer: OpenGLRenderer,
                joystick_subsystem: JoystickSubsystem,
                command_line_arguments: Arguments,
                //timer_subsystem: TimerSubsystem,
            ) -> Game {
        let mut game_logic = Logic::new();
        let quit = false;

        let mut audio_manager = if let & Some(ref music_file_path) = command_line_arguments.music_file_path() {
            AudioManager::new(music_file_path)
        } else {
            AudioManager::new("music.ogg")
        };

        let settings = Settings::new(&mut controller_subsystem, command_line_arguments);

        let input = InputManager::new(controller_subsystem, joystick_subsystem);
        let fps_counter = FpsCounter::new();
        let timer = GameLoopTimer::new(LOGIC_MAX_UPDATES_MILLISECONDS);

        let mut gui = GUI::new(&settings);
        gui.update_position_from_half_screen_width(renderer.half_screen_width_world_coordinates());


        settings.apply_current_settings(&mut renderer, &mut gui, &mut game_logic, &mut audio_manager);


        audio_manager.play_music();

        Game {
            game_logic,
            quit,
            input,
            fps_counter,
            timer,
            gui,
            renderer,
            settings,
            audio_manager,
            update_game: false,
            render_game: false,
            time_manager: TimeManager::new(),
        }
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => self.quit = true,
                Event::KeyDown {keycode: Some(key), ..} => self.input.update_key_down(key, self.time_manager.current_time()),
                Event::KeyUp {keycode: Some(key), ..} => self.input.update_key_up(key, self.time_manager.current_time()),
                Event::MouseMotion { x, y, ..} => self.input.update_mouse_motion(self.renderer.screen_coordinates_to_world_coordinates(x, y)),
                Event::MouseButtonUp { x, y, ..} =>  self.input.update_mouse_button_up(self.renderer.screen_coordinates_to_world_coordinates(x, y)),
                Event::ControllerDeviceRemoved { which, ..} => self.input.remove_game_controller(which),
                Event::ControllerAxisMotion { axis, value, ..} => self.input.game_controller_axis_motion(axis, value, self.time_manager.current_time()),
                Event::ControllerButtonDown { button, ..} => self.input.game_controller_button_down(button, self.time_manager.current_time()),
                Event::ControllerButtonUp { button, ..} => self.input.game_controller_button_up(button, self.time_manager.current_time()),
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
        self.fps_counter.frame();

        self.renderer.start();

        if self.render_game {
            self.renderer.render(&self.game_logic, false);
        } else {
            self.renderer.render(&self.game_logic, true);
        }

        self.renderer.render_gui(&self.gui);

        self.renderer.end();

        //std::thread::sleep(std::time::Duration::from_millis(50));
    }

    pub fn update(&mut self) {
        self.time_manager.update_time(self.update_game);

        let fps_updated = self.fps_counter.update(self.time_manager.current_time(), self.settings.print_fps_count());

        if fps_updated && self.gui.get_gui_fps_counter().show_fps() {
            self.gui.update_fps_counter(self.fps_counter.fps());
        }

        self.timer.update(self.time_manager.current_time());

        if self.timer.update_logic() {
            if self.update_game {
                self.game_logic.update(&self.input, &mut self.gui, self.audio_manager.sound_effect_manager_mut(), self.time_manager.game_time_manager());
            }

            match self.gui.handle_input(&mut self.input) {
                None => (),
                Some(GUIEvent::Exit) => self.quit = true,
                Some(GUIEvent::ChangeSetting(new_setting_value)) => {
                    self.settings.update_setting(new_setting_value);
                    Settings::apply_setting(new_setting_value, &mut self.renderer, &mut self.gui, &mut self.game_logic, &mut self.audio_manager);
                },
                Some(GUIEvent::NewGame(difficulty)) => {
                    self.game_logic.reset_game(&mut self.gui, difficulty, 0, self.time_manager.game_time_manager());
                    self.set_game_rendering_and_updating(true, true);
                },
                Some(GUIEvent::NextLevel) => {
                    self.game_logic.reset_to_next_level(&mut self.gui, self.time_manager.game_time_manager());
                    self.set_game_rendering_and_updating(true, true);
                },
                Some(GUIEvent::ChangeState(GUIState::Game)) => self.set_game_rendering_and_updating(true, true),
                Some(GUIEvent::ChangeState(GUIState::PauseMenu)) |
                Some(GUIEvent::ChangeState(GUIState::NextLevelScreen)) |
                Some(GUIEvent::ChangeState(GUIState::GameOverScreen)) |
                Some(GUIEvent::ChangeState(GUIState::PlayerWinsScreen)) => self.set_game_rendering_and_updating(true, false),
                Some(GUIEvent::ChangeState(_)) => self.set_game_rendering_and_updating(false, false),
            }

            self.input.update(self.time_manager.current_time());
            self.audio_manager.sound_effect_manager_mut().update();
        }
    }

    pub fn save_settings(&self) {
        self.settings.save();
    }

    pub fn set_game_rendering_and_updating(&mut self, rendering: bool, updating: bool) {
        self.render_game = rendering;
        self.update_game = updating;
    }
}