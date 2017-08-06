/*
src/settings.rs, 2017-08-06

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use gui::GUIEvent;

use std::env;
use std::fs::File;
use std::io::prelude::*;

use sdl2::GameControllerSubsystem;

use renderer::Renderer;
use gui::GUI;

#[derive(Copy, Clone, Debug)]
pub enum SettingEvent {
    FullScreen,
    ShowFpsCounter,
    VSync,
}

#[derive(Copy, Clone, Debug)]
pub enum SettingType {
    Boolean(SettingEvent, bool),
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
            SettingContainer::new("Full screen", SettingType::Boolean(SettingEvent::FullScreen, false)),
            SettingContainer::new("FPS counter", SettingType::Boolean(SettingEvent::ShowFpsCounter, false)),
            SettingContainer::new("VSync", SettingType::Boolean(SettingEvent::VSync, true)),
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

    pub fn event_from_index(&mut self, i: usize) -> Option<GUIEvent> {
        if i >= self.settings.len() {
            panic!("setting index out of bounds");
        } else {
            self.settings[i].update()
        }
    }

    pub fn save(&self) {
        use std::fmt::Write;

        let mut settings_text = String::new();

        settings_text.push_str("# Settings file for Space Boss Battles\n\n[Settings]\n");

        for setting in &self.settings {
            match setting.get_value() {
                SettingType::Boolean(_, value) => {
                    writeln!(settings_text, "{}={}", setting.get_name(), value).unwrap();
                }
            }
        }

        settings_text.push_str("\n[GameControllerMappings]\n# https://wiki.libsdl.org/SDL_GameControllerAddMapping\n\n");

        for mapping in &self.controller_mappings {
            settings_text.push_str(mapping);
            settings_text.push('\n');
        }

        let mut file = match File::create("settings.txt") {
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
        let mut file = match File::open("settings.txt") {
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

                    if !(value == "true" || value == "false") {
                        println!("couldn't load settings, invalid setting: {}", line);
                        continue;
                    }

                    for setting in &mut self.settings {
                        if setting.get_name() != name {
                            continue;
                        }

                        match setting.get_value() {
                            SettingType::Boolean(event, _) => {
                                if value == "true" {
                                    setting.set_value(SettingType::Boolean(event, true));
                                } else if value == "false" {
                                    setting.set_value(SettingType::Boolean(event, false));
                                }
                            },
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

    pub fn apply_current_settings<T: Renderer>(&self, renderer: &mut T, gui: &mut GUI) {
        for setting in &self.settings {
            self.apply_setting(setting.get_value(), renderer, gui);
        }
    }

    pub fn apply_setting<T: Renderer>(&self, setting: SettingType, renderer: &mut T, gui: &mut GUI) {
        match setting {
                SettingType::Boolean(event, value) => match event {
                    SettingEvent::FullScreen => {
                        renderer.full_screen(value);
                        gui.update_component_positions(renderer.half_screen_width_world_coordinates());
                    },
                    SettingEvent::ShowFpsCounter => gui.set_show_fps_counter(value),
                    SettingEvent::VSync => renderer.v_sync(value),
                },
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

    pub fn update(&mut self) -> Option<GUIEvent> {
        match self.setting_type {
            SettingType::Boolean(event, value) => self.setting_type = SettingType::Boolean(event, !value),
        }

        Some(GUIEvent::ChangeSetting(self.setting_type))
    }

    fn set_value(&mut self, value: SettingType) {
        self.setting_type = value;
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