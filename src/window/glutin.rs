

use std::os::raw::c_void;

use glutin::{EventsLoop, GlContext, WindowBuilder, ContextBuilder, GlWindow, GlRequest, Api, VirtualKeyCode};

use input::{InputManager, Key, Input};
use renderer::{Renderer, DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH};
use settings::Settings;
use gui::GUI;
use logic::Logic;
use utils::{TimeManager, TimeMilliseconds};
use audio::{Audio, Volume, AudioPlayer};

use super::{Window, RenderingContext, WINDOW_TITLE};


pub struct GlutinWindow {
    rendering_context: RenderingContext,
    events_loop: EventsLoop,
    window: GlWindow,
    mouse_x: i32,
    mouse_y: i32,
}


impl Window for GlutinWindow {
    type AudioPlayer = AudioPlayerRodio;

    fn new(rendering_context: RenderingContext) -> Result<Self, ()> {

        let events_loop = EventsLoop::new();
        let window_builder = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_dimensions(DEFAULT_SCREEN_WIDTH as u32, DEFAULT_SCREEN_HEIGHT as u32)
            .with_min_dimensions(DEFAULT_SCREEN_WIDTH as u32, DEFAULT_SCREEN_HEIGHT as u32);

        let gl_request = match rendering_context {
            RenderingContext::OpenGL => GlRequest::Specific(Api::OpenGl, (3,3)),
            RenderingContext::OpenGLES => GlRequest::Specific(Api::OpenGlEs, (2,0)),
        };

        let context_builder = ContextBuilder::new()
            .with_gl(gl_request)
            .with_vsync(true);
        let gl_window = match GlWindow::new(window_builder, context_builder, &events_loop) {
            Ok(window) => window,
            Err(error) => {
                println!("couldn't create window: {}", error);
                return Err(());
            }
        };

        unsafe {
            if let Err(error) = gl_window.make_current() {
                println!("couldn't make OpenGL context current: {}", error);
                return Err(());
            }
        }

        let window = Self {
            rendering_context,
            window: gl_window,
            events_loop,
            mouse_x: 0,
            mouse_y: 0,
        };

        Ok(window)
    }

    fn handle_events<R: Renderer>(
        &mut self,
        input_manager: &mut InputManager,
        renderer: &mut R,
        settings: &mut Settings,
        gui: &mut GUI,
        logic: &mut Logic,
        quit_flag: &mut bool,
        time_manager: &TimeManager,
    ) {
        use glutin::{Event, WindowEvent, KeyboardInput, ElementState};

        let mouse_x = &mut self.mouse_x;
        let mouse_y = &mut self.mouse_y;

        self.events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event: window_event, ..} => {
                    match window_event {
                        WindowEvent::Resized(width, height) => {
                            renderer.update_screen_size(width as i32, height as i32);
                            gui.update_position_from_half_screen_width(renderer.half_screen_width_world_coordinates());
                            logic.update_half_screen_width(renderer.half_screen_width_world_coordinates());
                        },
                        WindowEvent::Closed => *quit_flag = true,
                        WindowEvent::KeyboardInput {
                            input: KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                            ..
                        } => {
                            if let Some(key) = virtual_keycode_to_key(keycode) {
                                input_manager.update_key_down(key, time_manager.current_time());
                            }
                        }
                        WindowEvent::KeyboardInput {
                            input: KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                            ..
                        } => {
                            if let Some(key) = virtual_keycode_to_key(keycode) {
                                input_manager.update_key_up(key, time_manager.current_time());
                            }
                        }
                        WindowEvent::MouseInput { state: ElementState::Released, ..} => {
                            input_manager.update_mouse_button_up(renderer.screen_coordinates_to_world_coordinates(*mouse_x, *mouse_y));
                        },
                        WindowEvent::CursorMoved { position: (x, y), ..} => {
                            *mouse_x = x as i32;
                            *mouse_y = y as i32;

                            input_manager.update_mouse_motion(renderer.screen_coordinates_to_world_coordinates(*mouse_x, *mouse_y));
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        })
    }

    fn swap_buffers(&mut self) -> Result<(), ()> {
        self.window.swap_buffers().map_err(|error| {
            println!("couldn't swap buffers: {}", error);
        })
    }

    fn set_fullscreen(&mut self, value: bool) {
        if value {
            let current_monitor = self.window.get_current_monitor();
            self.window.set_fullscreen(Some(current_monitor));
        } else {
            self.window.set_fullscreen(None);
        }
    }

    fn set_v_sync(&mut self, value: bool) {
        // TODO: glutin window set v-sync setting at runtime
    }

    fn rendering_context(&self) -> RenderingContext {
        self.rendering_context
    }

    fn gl_get_proc_address(&self, function_name: &str) -> *const c_void {
        self.window.get_proc_address(function_name) as *const c_void
    }

    fn add_game_controller_mappings(&mut self, game_controller_mappings: &Vec<String>) {
        // TODO: glutin window game controller support
    }

    fn audio_player(&mut self) -> Option<Self::AudioPlayer> {
        // TODO: glutin window audio support
        None
    }
}

pub struct AudioPlayerRodio {

}

impl AudioPlayer for AudioPlayerRodio {
    type Music = AudioRodio;
    type Effect = AudioRodio;
}

pub struct AudioRodio {

}

impl Audio for AudioRodio {
    type Volume = VolumeRodio;

    fn load(file_path: &str) -> Result<Self, String> {
        unimplemented!()
    }

    fn play(&mut self) {
        unimplemented!()
    }

    fn change_volume(&mut self, volume: Self::Volume) {
        unimplemented!()
    }
}


#[derive(Debug, Clone, Copy)]
pub struct VolumeRodio {

}

impl Volume for VolumeRodio {
    type Value = i32;

    const MAX_VOLUME: Self::Value = 0;
    const DEFAULT_VOLUME_PERCENTAGE: i32 = 0;

    fn new(volume: Self::Value) -> Self {
        unimplemented!()
    }

    fn value(&self) -> Self::Value {
        unimplemented!()
    }

    fn from_percentage(percentage: i32) -> Self {
        let percentage = if percentage < 0 {
            0
        } else if 100 < percentage {
            100
        } else {
            percentage
        };

        VolumeRodio {}
    }
}

fn virtual_keycode_to_key(keycode: VirtualKeyCode) -> Option<Key> {
    let key = match keycode {
        VirtualKeyCode::Up    | VirtualKeyCode::W => Key::Up,
        VirtualKeyCode::Down  | VirtualKeyCode::S => Key::Down,
        VirtualKeyCode::Left  | VirtualKeyCode::A => Key::Left,
        VirtualKeyCode::Right | VirtualKeyCode::D => Key::Right,
        VirtualKeyCode::Space | VirtualKeyCode::LControl | VirtualKeyCode::RControl => Key::Shoot,
        VirtualKeyCode::Return => Key::Select,
        VirtualKeyCode::Escape  => Key::Back,
        _ => return None,
    };

    Some(key)
}