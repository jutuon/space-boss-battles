

use std::os::raw::c_void;


use sdl2::{EventPump, VideoSubsystem, GameControllerSubsystem, JoystickSubsystem};
use sdl2;

use sdl2::video::{FullscreenType, GLProfile, GLContext};

use sdl2::keyboard::Keycode;
use sdl2::controller::{GameController, Button, Axis};

use sdl2::mixer::{Channel, Chunk, Music};
use sdl2::mixer;


use input::{InputManager, Key, Input};
use renderer::{Renderer, DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH};
use settings::Settings;
use gui::GUI;
use logic::Logic;
use utils::{TimeManager, TimeMilliseconds};
use audio::{Audio, Volume, AudioPlayer};

use super::{Window, RenderingContext};

#[cfg(not(target_os = "emscripten"))]
const PAUSE_KEY: Keycode = Keycode::Escape;

// Web browser will exit from full screen mode with escape key, so there
// needs to be different key for pausing the game.
#[cfg(target_os = "emscripten")]
const PAUSE_KEY: Keycode = Keycode::P;


pub struct SDL2Window {
    video_subsystem: VideoSubsystem,
    event_pump: EventPump,
    rendering_context: RenderingContext,
    game_controller_manager: GameControllerManager,
    window: sdl2::video::Window,
    /// OpenGL context is stored here because it
    /// would be otherwise dropped.
    _context: GLContext,
    audio_player: Option<AudioPlayerSDL2>,
}

impl Window for SDL2Window {
    type AudioPlayer = AudioPlayerSDL2;

    fn new(rendering_context: RenderingContext) -> Result<Self,()> {
        let sdl_context = sdl2::init().expect("sdl2 init failed");
        println!("SDL2 version: {}", sdl2::version::version());

        let event_pump = sdl_context.event_pump().expect("failed to get handle to sdl2 event_pump");

        let game_controller_subsystem = sdl_context.game_controller().expect("game controller subsystem init failed");
        let joystick_subsystem = sdl_context.joystick().expect("joystick subsystem init failed");

        let video_subsystem = sdl_context.video().expect("video subsystem init fail");

        let window = video_subsystem.window("Space Boss Battles", DEFAULT_SCREEN_WIDTH as u32, DEFAULT_SCREEN_HEIGHT as u32).opengl().build().expect("window creation failed");

        match rendering_context {
            RenderingContext::OpenGL => {
                let gl_attr = video_subsystem.gl_attr();
                gl_attr.set_context_profile(GLProfile::Core);
                gl_attr.set_context_version(3,3);
            }
            RenderingContext::OpenGLES => {
                let gl_attr = video_subsystem.gl_attr();
                gl_attr.set_context_profile(GLProfile::GLES);
                gl_attr.set_context_version(2,0);
            }
        }

        let _context = window.gl_create_context().expect("opengl context creation failed");
        window.gl_make_current(&_context).expect("couldn't set opengl context to current thread");

        let window = Self {
            event_pump,
            video_subsystem,
            rendering_context,
            game_controller_manager: GameControllerManager::new(joystick_subsystem, game_controller_subsystem),
            window,
            _context,
            audio_player: AudioPlayerSDL2::new(),
        };

        Ok(window)
    }

    fn handle_events<R: Renderer>(
        &mut self,
        input: &mut InputManager,
        renderer: &mut R,
        settings: &mut Settings,
        gui: &mut GUI,
        logic: &mut Logic,
        quit_flag: &mut bool,
        time_manager: &TimeManager,
    ) {
        use sdl2::event::{Event, WindowEvent};

        for event in self.event_pump.poll_iter() {
            match event {
                    Event::Quit {..} => *quit_flag = true,
                    Event::KeyDown {keycode: Some(keycode), ..} => {
                        if let Some(key) = keycode_to_key(keycode) {
                            input.update_key_down(key, time_manager.current_time())
                        }
                    }
                    Event::KeyUp {keycode: Some(keycode), ..} => {
                        if let Some(key) = keycode_to_key(keycode) {
                            input.update_key_up(key, time_manager.current_time());
                        }
                    }
                    Event::MouseMotion { x, y, ..} => input.update_mouse_motion(renderer.screen_coordinates_to_world_coordinates(x, y)),
                    Event::MouseButtonUp { x, y, ..} =>  input.update_mouse_button_up(renderer.screen_coordinates_to_world_coordinates(x, y)),
                    Event::ControllerDeviceRemoved { which, ..} => self.game_controller_manager.remove_game_controller(which),
                    Event::ControllerAxisMotion { axis, value, ..} => GameControllerManager::handle_axis_motion(axis, value, input, time_manager.current_time()),
                    Event::ControllerButtonDown { button, ..} => {
                        if let Button::A = button {
                            input.update_key_down(Key::Select, time_manager.current_time());
                        }

                        if let Some(key) = GameControllerManager::button_to_key(button) {
                            input.update_key_down(key, time_manager.current_time());
                        }
                    },
                    Event::ControllerButtonUp { button, ..} => {
                        if let Button::A = button {
                            input.update_key_up(Key::Select, time_manager.current_time());
                        }

                        if let Some(key) = GameControllerManager::button_to_key(button) {
                            input.update_key_up(key, time_manager.current_time());
                        }
                    },
                    Event::JoyDeviceAdded { which, ..} => {
                        if let Some(mapping) = self.game_controller_manager.add_game_controller_from_joystick_id(which) {
                            settings.add_game_controller_mapping(mapping);
                        }
                    },
                    Event::Window { win_event: WindowEvent::SizeChanged(window_width_pixels, window_height_pixels), ..} => {

                        #[cfg(target_os = "emscripten")]
                        {
                            // It seems that full screen setting on emscripten build does not always change the game to full screen mode, so
                            // lets set full screen mode to disabled when event's screen width is less or equal than current screen width.
                            if window_width_pixels <= self.renderer.screen_width_pixels() {
                                let value = false;
                                let setting = settings::BooleanSetting::FullScreen;
                                settings.update_setting(settings::SettingType::Boolean(setting, value));
                                gui.get_settings_menu().set_boolean_setting(setting, value);
                            }
                        }

                        renderer.update_screen_size(window_width_pixels, window_height_pixels);
                        gui.update_position_from_half_screen_width(renderer.half_screen_width_world_coordinates());
                        logic.update_half_screen_width(renderer.half_screen_width_world_coordinates());
                    },
                    _ => (),
            }

            if settings.print_joystick_events() {
                match event {
                    Event::JoyAxisMotion { value, axis_idx, .. } => println!("JoyAxisMotion, value: {}, axis_idx: {},", value, axis_idx),
                    Event::JoyBallMotion { ball_idx, xrel, yrel, .. } => println!("JoyBallMotion, ball_idx: {}, xrel: {}, yrel: {}", ball_idx, xrel, yrel),
                    Event::JoyHatMotion { hat_idx, state, .. } => println!("JoyHatMotion, hat_idx: {}, state as number: {}, state: {:?}", hat_idx, state as u32, state),
                    Event::JoyButtonDown { button_idx, .. } => println!("JoyButtonDown, button_idx: {}", button_idx),
                    _ => (),
                }
            }
        }
    }


    fn swap_buffers(&mut self) -> Result<(), ()> {
        self.window.gl_swap_window();

        Ok(())
    }

    /// Enable or disable full screen mode.
    fn set_fullscreen(&mut self, value: bool) {
        let setting;

        if value {
            setting = FullscreenType::Desktop;
        } else {
            setting = FullscreenType::Off;
        }

        if let Err(message) = self.window.set_fullscreen(setting) {
            println!("Error, couldn't change fullscreen setting: {}", message);
        }
    }

    /// Enable or disable vertical synchronization.
    fn set_v_sync(&mut self, value: bool) {
        if value {
            self.video_subsystem.gl_set_swap_interval(1);
        } else {
            self.video_subsystem.gl_set_swap_interval(0);
        }
    }

    fn rendering_context(&self) -> RenderingContext {
        self.rendering_context
    }

    fn gl_get_proc_address(&self, function_name: &str) -> *const c_void {
        self.video_subsystem.gl_get_proc_address(function_name) as *const c_void
    }

    fn add_game_controller_mappings(&mut self, game_controller_mappings: &Vec<String>) {
        for mapping in game_controller_mappings {
            if let Err(error) = self.game_controller_manager.game_controller_subsystem.add_mapping(mapping) {
                println!("error when loading game controller mapping \"{}\", error: {}", mapping, error);
            }
        }
    }

    fn audio_player(&mut self) -> Option<Self::AudioPlayer> {
        self.audio_player.take()
    }
}



fn keycode_to_key(keycode: Keycode) -> Option<Key> {
    let key = match keycode {
        Keycode::Up | Keycode::W => Key::Up,
        Keycode::Down | Keycode::S => Key::Down,
        Keycode::Left | Keycode::A => Key::Left,
        Keycode::Right | Keycode::D => Key::Right,
        Keycode::Space | Keycode::LCtrl | Keycode::RCtrl => Key::Shoot,
        Keycode::Return => Key::Select,
        PAUSE_KEY  => Key::Back,
        _ => return None,
    };

    Some(key)
}


type GameControllerMapping = String;

/// Add and remove game controllers, route game controller events to `KeyboardManager`
struct GameControllerManager {
    joystick_subsystem: JoystickSubsystem,
    game_controller_subsystem: GameControllerSubsystem,
    game_controllers: Vec<GameController>,
}

impl GameControllerManager {
    /// Create new `GameControllerManager`
    fn new(joystick_subsystem: JoystickSubsystem, game_controller_subsystem: GameControllerSubsystem) -> GameControllerManager {
        GameControllerManager {
            joystick_subsystem,
            game_controller_subsystem,
            game_controllers: Vec::new(),
        }
    }

    /// Adds new game controller from SDL2 joystick id to `GameControllerManager`.
    ///
    /// If the joystick doesn't have a game controller mapping, method will create default
    /// mapping for the joystick and return the created mapping.
    ///
    /// If there is an error it will be printed to standard output.
    pub fn add_game_controller_from_joystick_id(&mut self, id: u32) -> Option<GameControllerMapping> {
        let game_controller_mapping = if !self.game_controller_subsystem.is_game_controller(id) {
            let joystick_name;
            match self.joystick_subsystem.name_for_index(id) {
                Ok(name) => joystick_name = name,
                Err(error) => {
                    println!("error: {}", error);
                    return None;
                }
            }

            let mut joystick_guid;
            match self.joystick_subsystem.device_guid(id) {
                Ok(guid) => joystick_guid = guid.to_string(),
                Err(error) => {
                    println!("error: {}", error);
                    return None;
                }
            }

            // https://wiki.libsdl.org/SDL_GameControllerAddMapping
            joystick_guid.push(',');
            joystick_guid.push_str(&joystick_name);
            joystick_guid.push_str(", a:b2, b:b1, y:b0, x:b3, start:b9, guide:b12, back:b8, dpup:h0.1, dpleft:h0.8, dpdown:h0.4, dpright:h0.2, leftshoulder:b6, rightshoulder:b7, leftstick:b10, rightstick:b11, leftx:a0, lefty:a1, rightx:a3, righty:a2, lefttrigger:b4, righttrigger:b5");

            match self.game_controller_subsystem.add_mapping(&joystick_guid) {
                Ok(_) => {
                    println!("default game controller mapping loaded for joystick with id {}", id);
                    Some(joystick_guid)
                },
                Err(error) => {
                    println!("error: {}", error);
                    return None
                }
            }
        } else {
            None
        };

        match self.game_controller_subsystem.open(id) {
            Ok(controller) => {
                self.game_controllers.push(controller);
                println!("game controller with id {} added", id);
            },
            Err(integer_or_sdl_error) => println!("game controller error: {}", integer_or_sdl_error),
        }

        game_controller_mapping
    }

    /// Remove game controller which has same id as argument `id`.
    ///
    /// Game controller will be removed from `GameControllerManager`'s `Vec<GameController>`
    pub fn remove_game_controller(&mut self, id: i32) {
        let mut index = None;

        for (i, controller) in self.game_controllers.iter().enumerate() {
            if controller.instance_id() == id {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            self.game_controllers.swap_remove(i);
            println!("game controller with id {} removed", id);
        }
    }


    /// Forwards game controller's axis event to `InputManager`.
    pub fn handle_axis_motion(axis: Axis, value: i16, input_manager: &mut InputManager, current_time: &TimeMilliseconds) {
        match axis {
            Axis::LeftX | Axis::RightX => {
                if value > 10000 {
                    input_manager.update_key_down(Key::Right, current_time);
                } else if value < -10000 {
                    input_manager.update_key_down(Key::Left, current_time);
                } else {
                    if input_manager.left() {
                        input_manager.update_key_up(Key::Left, current_time);
                    }
                    if input_manager.right() {
                        input_manager.update_key_up(Key::Right, current_time);
                    }
                }
            },
            Axis::LeftY | Axis::RightY => {
                if value > 10000 {
                    input_manager.update_key_down(Key::Down, current_time);
                } else if value < -10000 {
                    input_manager.update_key_down(Key::Up, current_time);
                } else {
                    if input_manager.down() {
                        input_manager.update_key_up(Key::Down, current_time);
                    }
                    if input_manager.up() {
                        input_manager.update_key_up(Key::Up, current_time);
                    }
                }
            },
            Axis::TriggerLeft | Axis::TriggerRight => {
                if value > 100 {
                    input_manager.update_key_down(Key::Shoot, current_time);
                } else {
                    input_manager.update_key_up(Key::Shoot, current_time);
                }
            },
        }
    }

    pub fn button_to_key(button: Button) -> Option<Key> {
        let key = match button {
            Button::DPadUp     => Key::Up,
            Button::DPadDown   => Key::Down,
            Button::DPadLeft   => Key::Left,
            Button::DPadRight  => Key::Right,
            Button::A | Button::LeftShoulder | Button::RightShoulder => Key::Shoot,
            Button::Back       => Key::Back,
            _ => return None,
        };

        Some(key)
    }
}


/// Sound effect's audio data and current `sdl2::mixer::Channel`
pub struct SoundEffectSDL2 {
    channel: Channel,
    chunk: Chunk,
}

impl Audio for SoundEffectSDL2 {
    type Volume = VolumeSDL2;

    /// Load new sound effect.
    fn load(file_path: &str) -> Result<Self, String> {
        let sound_effect = Self {
            channel: Channel::all(),
            chunk: Chunk::from_file(file_path)?,
        };

        Ok(sound_effect)
    }

    /// Play sound effect.
    ///
    /// Prints error message to standard output if there is sound effect
    /// playing error.
    fn play(&mut self) {
        self.channel = match self.channel.play(&self.chunk, 0) {
            Ok(channel) => channel,
            Err(message) => {
                println!("sound effect playing error: {}", message);
                Channel::all()
            },
        };
    }

    /// Change sound effect's volume.
    fn change_volume(&mut self, volume: Self::Volume) {
        self.chunk.set_volume(volume.value());
    }
}

/// Wrapper for correct audio volume value.
#[derive(Copy, Clone)]
pub struct VolumeSDL2(i32);

impl Volume for VolumeSDL2 {
    type Value = i32;

    const MAX_VOLUME: Self::Value = mixer::MAX_VOLUME;
    const DEFAULT_VOLUME_PERCENTAGE: i32 = 68;

    /// Create new volume value limited to [0; MAX_VOLUME].
    fn new(volume: Self::Value) -> Self {
        if volume > Self::MAX_VOLUME {
            VolumeSDL2(Self::MAX_VOLUME)
        } else if volume < 0 {
            VolumeSDL2(0)
        } else {
            VolumeSDL2(volume)
        }
    }

    /// Get volume value.
    fn value(&self) -> Self::Value {
        self.0
    }

    fn from_percentage(percentage: i32) -> Self {
        let percentage = if percentage < 0 {
            0
        } else if 100 < percentage {
            100
        } else {
            percentage
        };

        let volume = Self::MAX_VOLUME as f32 * (percentage as f32 / 100.0);

        Self::new(volume as i32)
    }
}


pub struct MusicSDL2 {
    music: Music<'static>,
}

impl Audio for MusicSDL2 {
    type Volume = VolumeSDL2;

    /// Load music from file.
    fn load(music_file_path: &str) -> Result<Self, String> {
        let music_wrapper = Self {
            music: Music::from_file(music_file_path)?,
        };

        Ok( music_wrapper )
    }

    /// Set music volume.
    fn change_volume(&mut self, volume: Self::Volume) {
        Music::set_volume(volume.value());
    }

    /// Start playing music if it isn't already playing.
    ///
    /// If starting the music failed, an error message will
    /// be printed to the standard output.
    fn play(&mut self) {
        if !Music::is_playing() {
            if let Err(message) = self.music.play(-1) {
                println!("music error: {}", message);
            }
        }
    }
}


pub struct AudioPlayerSDL2;

impl AudioPlayerSDL2 {
    pub fn new() -> Option<Self> {
        if let Err(error) = mixer::open_audio(44100, mixer::DEFAULT_FORMAT, mixer::DEFAULT_CHANNELS, 1024) {
            println!("SDL_mixer init error: {}", error);

            None
        } else {
            println!("SDL_mixer version: {}", mixer::get_linked_version());

            Some(AudioPlayerSDL2)
        }
    }
}

impl AudioPlayer for AudioPlayerSDL2 {
    type Music = MusicSDL2;
    type Effect = SoundEffectSDL2;
}

impl Drop for AudioPlayerSDL2 {
    /// Call function `sdl2::mixer::close_audio`.
    fn drop(&mut self) {
        mixer::close_audio();
    }
}