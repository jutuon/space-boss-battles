/*
src/main.rs, 2017-09-02

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Source code for Space Boss Battles.
//!
//! Main function and game loop is in this file.

#[cfg(not(feature = "glutin_window"))]
extern crate sdl2;

#[cfg(feature = "glutin_window")]
extern crate glutin;

extern crate gl;
extern crate image;
extern crate cgmath;
extern crate rand;

#[cfg(target_os = "emscripten")]
extern crate emscripten_sys;

pub mod gui;
pub mod logic;
pub mod renderer;
pub mod input;
pub mod settings;
pub mod audio;
pub mod utils;
pub mod window;

use std::env;

use renderer::{Renderer, OpenGLRenderer};
use logic::Logic;

use input::{InputManager};
use gui::{GUI, GUIEvent, GUIState};

use settings::{Settings, Arguments};

use audio::{AudioManager, SoundEffectPlayer, AudioPlayer, Audio, Volume};

use utils::{FpsCounter, GameLoopTimer, TimeManager};

use window::{Window, RenderingContext};

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

/// Check command line arguments, initialize game and start game loop.
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


    #[cfg(not(feature = "gles"))]
    let rendering_context = RenderingContext::OpenGL;

    #[cfg(feature = "gles")]
    let rendering_context = RenderingContext::OpenGLES;

    #[cfg(not(feature = "glutin_window"))]
    let window = window::sdl2::SDL2Window::new(rendering_context).expect("window creation failed");

    #[cfg(feature = "glutin_window")]
    let window = window::glutin::GlutinWindow::new(rendering_context).expect("window creation failed");

    let mut game = Game::new(arguments, window);

    #[cfg(target_os = "emscripten")]
    {
        let game_ptr: *mut Game = &mut game;

        unsafe {
            // This function will not return because last parameter named `simulate_infinite_loop` is true.
            // Function documentation: https://kripken.github.io/emscripten-site/docs/api_reference/emscripten.h.html#c.emscripten_set_main_loop_arg
            emscripten_sys::emscripten_set_main_loop_arg(Some(game_loop_iteration_emscripten), game_ptr as *mut std::os::raw::c_void, 0, 1);
        }
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        loop {
            if game.quit() {
                game.save_settings();
                break;
            }

            game.handle_events();

            game.update();

            game.render();
        }
    }
}

/// One iteration of game loop for emscripten build.
#[cfg(target_os = "emscripten")]
unsafe extern fn game_loop_iteration_emscripten(game: *mut std::os::raw::c_void) {
    let game = game as *mut Game;

    // Quit button is disabled in emscripten build, so this is true only if user closes the web page.
    if (*game).quit() {
        /// TODO: Emscripten build does not support saving the settings.

        // Drop game to free resources.
        std::ptr::drop_in_place(game);
        return;
    }

    (*game).handle_events();

    (*game).update();

    (*game).render();
}

/// Store game components and handle interaction between all components.
pub struct Game<W: Window> {
    game_logic: Logic,
    quit: bool,
    input: InputManager,
    fps_counter: FpsCounter,
    timer: GameLoopTimer,
    gui: GUI,
    renderer: OpenGLRenderer,
    settings: Settings,
    audio_manager: AudioManager<W::AudioPlayer>,
    update_game: bool,
    render_game: bool,
    time_manager: TimeManager,
    window: W,
}

impl<W: Window> Game<W> {
    /// Create new `Game`. Creates and initializes game's components.
    pub fn new(
                command_line_arguments: Arguments,
                mut window: W,
            ) -> Self {

        let player = window.audio_player();

        let mut audio_manager = if let & Some(ref music_file_path) = command_line_arguments.music_file_path() {
            AudioManager::new(music_file_path, player)
        } else {
            AudioManager::new("music.ogg", player)
        };

        let settings = Settings::new(
            command_line_arguments,
            <<<W::AudioPlayer as AudioPlayer>::Effect as Audio>::Volume as Volume>::DEFAULT_VOLUME_PERCENTAGE,
            <<<W::AudioPlayer as AudioPlayer>::Music as Audio>::Volume as Volume>::DEFAULT_VOLUME_PERCENTAGE,
        );

        window.add_game_controller_mappings(settings.game_controller_mappings());

        let input = InputManager::new();

        let mut renderer = OpenGLRenderer::new(&window);
        let mut gui = GUI::new(&settings);
        gui.update_position_from_half_screen_width(renderer.half_screen_width_world_coordinates());

        let mut game_logic = Logic::new();
        game_logic.update_half_screen_width(renderer.half_screen_width_world_coordinates());

        settings.apply_current_settings(&mut renderer, &mut gui, &mut audio_manager, &mut window);

        // Try to play music after getting audio volume from settings.
        audio_manager.play_music();

        Game {
            game_logic,
            quit: false,
            input,
            fps_counter: FpsCounter::new(),
            timer: GameLoopTimer::new(LOGIC_MAX_UPDATES_MILLISECONDS),
            gui,
            renderer,
            settings,
            audio_manager,
            update_game: false,
            render_game: false,
            time_manager: TimeManager::new(),
            window,
        }
    }

    /// Return true if game should be closed.
    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn handle_events(&mut self) {
        self.window.handle_events(
            &mut self.input,
            &mut self.renderer,
            &mut self.settings,
            &mut self.gui,
            &mut self.game_logic,
            &mut self.quit,
            &self.time_manager
        );
    }

    /// Render game's current state.
    pub fn render(&mut self) {
        self.fps_counter.frame();

        self.renderer.start();

        if self.render_game {
            self.renderer.render(&self.game_logic, false);
        } else {
            self.renderer.render(&self.game_logic, true);
        }

        self.renderer.render_gui(&self.gui);

        self.renderer.end(&mut self.window);
    }

    /// Updates logic and other game components.
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
                    Settings::apply_setting(new_setting_value, &mut self.renderer, &mut self.gui, &mut self.audio_manager, &mut self.window);
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

    /// Save current settings.
    pub fn save_settings(&self) {
        self.settings.save();
    }

    /// Set game logic rendering and updating options.
    pub fn set_game_rendering_and_updating(&mut self, rendering: bool, updating: bool) {
        self.render_game = rendering;
        self.update_game = updating;
    }
}