/*
src/settings.rs, 2017-08-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use std::env;
use std::fs::File;
use std::io::prelude::*;

use sdl2::GameControllerSubsystem;

use renderer::Renderer;

use gui::GUI;
use gui::components::GUIUpdatePosition;

use logic::Logic;

use audio::AudioManager;

const SETTINGS_FILE_NAME: &'static str = "space_boss_battles_settings.txt";

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IntegerSetting {
    SoundEffectVolume,
    MusicVolume,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BooleanSetting {
    FullScreen,
    ShowFpsCounter,
    VSync,
}

#[derive(Copy, Clone, Debug)]
pub enum SettingType {
    Boolean(BooleanSetting, bool),
    Integer(IntegerSetting, i32),
}

pub struct Settings {
    settings: Vec<SettingContainer>,
    controller_mappings: Vec<String>,
    print_fps_count: bool,
    print_joystick_events: bool,
}

impl Settings {
    pub fn new(game_controller_subsystem: &mut GameControllerSubsystem) -> Settings {
        let settings = vec![
            SettingContainer::new("Full screen", SettingType::Boolean(BooleanSetting::FullScreen, false)),
            SettingContainer::new("FPS counter", SettingType::Boolean(BooleanSetting::ShowFpsCounter, false)),
            SettingContainer::new("VSync", SettingType::Boolean(BooleanSetting::VSync, true)),
            SettingContainer::new("Music volume", SettingType::Integer(IntegerSetting::MusicVolume, AudioManager::max_volume())),
            SettingContainer::new("Effect volume", SettingType::Integer(IntegerSetting::SoundEffectVolume, AudioManager::max_volume())),

        ];

        let mut settings = Settings {
            settings: settings,
            controller_mappings: Vec::new(),
            print_fps_count: false,
            print_joystick_events: false,
        };

        settings.load();
        settings.load_game_controller_mappings(game_controller_subsystem);
        settings.handle_command_line_arguments();

        settings
    }

    pub fn get_settings(&self) -> &Vec<SettingContainer> {
        &self.settings
    }

    pub fn update_setting(&mut self, new_value: SettingType) {
        // FIXME: Change Vec<SettingContainer> to better system, so there won't
        //        be need to find correct setting with loop.

        match new_value {
            SettingType::Boolean(event, value) => {
                for setting in &mut self.settings {
                    if setting.set_if_boolean_setting_matches(event, value) {
                        return;
                    }
                }
            },
            SettingType::Integer(event, value) => {
                for setting in &mut self.settings {
                    if setting.set_if_integer_setting_matches(event, value) {
                        return;
                    }
                }
            },
        }

        println!("unimplemented setting found: {:?}", new_value);
    }

    pub fn save(&self) {
        use std::fmt::Write;

        let mut settings_text = String::new();

        settings_text.push_str("# Settings file for Space Boss Battles\n\n[Settings]\n");

        for setting in &self.settings {
            match setting.get_value() {
                SettingType::Boolean(_, value) => {
                    writeln!(settings_text, "{}={}", setting.get_name(), value).unwrap();
                },
                SettingType::Integer(_, value) => {
                    writeln!(settings_text, "{}={}", setting.get_name(), value).unwrap();
                }
            }
        }

        settings_text.push_str("\n[GameControllerMappings]\n# https://wiki.libsdl.org/SDL_GameControllerAddMapping\n\n");

        for mapping in &self.controller_mappings {
            settings_text.push_str(mapping);
            settings_text.push('\n');
        }

        let mut file = match File::create(SETTINGS_FILE_NAME) {
            Ok(file) => file,
            Err(error) => {
                println!("couldn't save settings: {}", error);
                return;
            }
        };

        if let Err(error) = file.write_all(settings_text.as_bytes()) {
            println!("couldn't save settings: {}", error);
        }
    }

    pub fn load(&mut self) {
        let mut file = match File::open(SETTINGS_FILE_NAME) {
            Ok(file) => file,
            Err(error) => {
                println!("couldn't load settings: {}", error);
                return;
            },
        };

        let mut settings_text = String::new();

        if let Err(error) = file.read_to_string(&mut settings_text) {
            println!("couldn't load settings: {}", error);
            return;
        }

        let mut settings_parser = SettingsParserMode::None;

        for line in settings_text.lines() {
            let line = line.trim();

            if line == "" || line.starts_with("#") {
                continue;
            } else if line == "[Settings]" {
                settings_parser = SettingsParserMode::Settings;
                continue;
            } else if line == "[GameControllerMappings]" {
                settings_parser = SettingsParserMode::GameControllerMappings;
                continue;
            }

            match settings_parser {
                SettingsParserMode::Settings => {
                    let mut iterator = line.split("=");
                    let name = match iterator.next() {
                        Some(name) => name,
                        None => {
                            println!("couldn't load settings, invalid setting: {}", line);
                            continue;
                        }
                    };

                    let value = match iterator.next() {
                        Some(name) => name,
                        None => {
                            println!("couldn't load settings, invalid setting: {}", line);
                            continue;
                        }
                    };

                    for setting in &mut self.settings {
                        if setting.get_name() != name {
                            continue;
                        }

                        match setting.get_value() {
                            SettingType::Boolean(event, _) => {
                                if value == "true" {
                                    setting.set_if_boolean_setting_matches(event, true);
                                } else if value == "false" {
                                    setting.set_if_boolean_setting_matches(event, false);
                                } else {
                                    println!("error when parsing value \"{}\" for setting \"{}\": not a boolean value", value, setting.get_name());
                                }
                            },
                            SettingType::Integer(event, _) => {
                                match value.parse::<i32>() {
                                    Ok(number) => {
                                        setting.set_if_integer_setting_matches(event, number);
                                    },
                                    Err(error) => println!("error when parsing value \"{}\" for setting \"{}\": {}", value, setting.get_name(), error),
                                }
                            }
                        }
                    }

                },
                SettingsParserMode::GameControllerMappings => {
                    self.controller_mappings.push(line.to_string());
                },
                _ => (),
            }
        }
    }

    pub fn load_game_controller_mappings(&self, controller_system: &mut GameControllerSubsystem) {
        for mapping in &self.controller_mappings {
            if let Err(error) = controller_system.add_mapping(mapping) {
                println!("error when loading game controller mapping \"{}\", error: {}", mapping, error);
            }
        }
    }

    pub fn add_game_controller_mapping(&mut self, mapping: String) {
        self.controller_mappings.push(mapping);
    }

    pub fn print_joystick_events(&self) -> bool {
        self.print_joystick_events
    }

    pub fn print_fps_count(&self) -> bool {
        self.print_fps_count
    }

    fn handle_command_line_arguments(&mut self) {
        use COMMAND_LINE_HELP_TEXT;

        let args = env::args();

        for arg in args.skip(1) {
            if arg == "--fps" {
                self.print_fps_count = true;
            } else if arg == "--joystick-events" {
                self.print_joystick_events = true;
            } else {
                println!("unknown argument: {}", arg);
                println!("{}", COMMAND_LINE_HELP_TEXT);
            }
        }
    }

    pub fn apply_current_settings<T: Renderer>(&self, renderer: &mut T, gui: &mut GUI, game_logic: &mut Logic, audio_manager: &mut AudioManager) {
        for setting in &self.settings {
            self.apply_setting(setting.get_value(), renderer, gui, game_logic, audio_manager);
        }
    }

    pub fn apply_setting<T: Renderer>(&self, setting: SettingType, renderer: &mut T, gui: &mut GUI, game_logic: &mut Logic, audio_manager: &mut AudioManager) {
        match setting {
            SettingType::Boolean(BooleanSetting::FullScreen, value) => {
                    renderer.full_screen(value);
                    gui.update_position_from_half_screen_width(renderer.half_screen_width_world_coordinates());
                    game_logic.update_half_screen_width(renderer.half_screen_width_world_coordinates());
            },
            SettingType::Boolean(BooleanSetting::ShowFpsCounter, value) => gui.set_show_fps_counter(value),
            SettingType::Boolean(BooleanSetting::VSync , value)  => renderer.v_sync(value),
            SettingType::Integer(IntegerSetting::SoundEffectVolume, value) => audio_manager.sound_effect_manager_mut().change_volume(value),
            SettingType::Integer(IntegerSetting::MusicVolume, value) => audio_manager.set_music_volume(value),
        }
    }
}

enum SettingsParserMode {
    None,
    Settings,
    GameControllerMappings,
}

pub trait Setting {
    fn get_name(&self) -> &str;
    fn get_value(&self) -> SettingType;
}

pub struct SettingContainer {
    name: &'static str,
    setting_type: SettingType,
}

impl SettingContainer {
    pub fn new(name: &'static str, setting_type: SettingType) -> SettingContainer {
        SettingContainer { name, setting_type }
    }

    fn set_if_boolean_setting_matches(&mut self, setting: BooleanSetting, value: bool) -> bool {
        if let &mut SettingType::Boolean(container_setting, ref mut old_value) = &mut self.setting_type {
            if container_setting == setting {
                *old_value = value;
                return true;
            }
        }

        false
    }

    fn set_if_integer_setting_matches(&mut self, setting: IntegerSetting, value: i32) -> bool {
        if let &mut SettingType::Integer(container_setting, ref mut old_value) = &mut self.setting_type {
            if container_setting == setting {
                *old_value = value;
                return true;
            }
        }

        false
    }
}

impl Setting for SettingContainer {
    fn get_name(&self) -> &str {
        self.name
    }

    fn get_value(&self) -> SettingType {
        self.setting_type
    }
}