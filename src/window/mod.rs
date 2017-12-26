pub mod sdl2;


use std::os::raw::c_void;

use input::InputManager;
use renderer::Renderer;
use settings::Settings;
use gui::GUI;
use logic::Logic;
use utils::TimeManager;
use audio::AudioPlayer;

#[derive(Debug, Clone, Copy)]
pub enum RenderingContext {
    OpenGL,
    OpenGLES,
}

pub trait Window: Sized {
    type AudioPlayer: AudioPlayer;

    fn new(RenderingContext) -> Result<Self, ()>;

    fn handle_events<R: Renderer>(
        &mut self,
        &mut InputManager,
        &mut R,
        &mut Settings,
        &mut GUI,
        &mut Logic,
        quit_flag: &mut bool,
        &TimeManager,
    );

    fn swap_buffers(&mut self) -> Result<(), ()>;

    fn set_fullscreen(&mut self, bool);

    fn set_v_sync(&mut self, bool);

    fn rendering_context(&self) -> RenderingContext;

    fn gl_get_proc_address(&self, &str) -> *const c_void;

    fn add_game_controller_mappings(&mut self, &Vec<String>);

    fn audio_player(&mut self) -> Option<Self::AudioPlayer>;
}