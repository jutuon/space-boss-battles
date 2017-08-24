/*
src/settings.rs, 2017-08-24

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Settings loading and saving, command line arguments.

use std::env::Args;
use std::fs::File;
use std::io::prelude::*;

use sdl2::GameControllerSubsystem;

use renderer::Renderer;

use gui::GUI;

use audio::{AudioManager, Volume};
use audio;

const SETTINGS_FILE_NAME: &'static str = "space_boss_battles_settings.txt";

/// Settings with integer value.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IntegerSetting {
    SoundEffectVolume,
    MusicVolume,
}

/// Settings with boolean value.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BooleanSetting {
    FullScreen,
    ShowFpsCounter,
    VSync,
}

/// Setting and it's value.
#[derive(Copy, Clone, Debug)]
pub enum SettingType {
    Boolean(BooleanSetting, bool),
    Integer(IntegerSetting, i32),
}

/// Save and load settings. Handle command line argument settings.
pub struct Settings {
    settings: Vec<SettingContainer>,
    controller_mappings: Vec<String>,
    command_line_arguments: Arguments,
}

impl Settings {
    /// Create new `Settings`.
    ///
    /// Read settings from file and load found game controller mappings to
    /// `GameControllerSubsystem`.
    pub fn new(game_controller_subsystem: &mut GameControllerSubsystem, command_line_arguments: Arguments) -> Settings {
        let settings = vec![
            SettingContainer::new("Full screen", SettingType::Boolean(BooleanSetting::FullScreen, false)),
            SettingContainer::new("FPS counter", SettingType::Boolean(BooleanSetting::ShowFpsCounter, false)),
            SettingContainer::new("VSync", SettingType::Boolean(BooleanSetting::VSync, true)),
            SettingContainer::new("Music volume", SettingType::Integer(IntegerSetting::MusicVolume, audio::DEFAULT_VOLUME)),
            SettingContainer::new("Effect volume", SettingType::Integer(IntegerSetting::SoundEffectVolume, audio::DEFAULT_VOLUME)),

        ];

        let mut settings = Settings {
            settings: settings,
            controller_mappings: Vec::new(),
            command_line_arguments,
        };

        settings.load();
        settings.load_game_controller_mappings(game_controller_subsystem);

        settings
    }

    /// Get settings.
    pub fn get_settings(&self) -> &Vec<SettingContainer> {
        &self.settings
    }

    /// Updates new value to `SettingContainer` existing in field `Vec<SettingContainer>`.
    ///
    /// Update will only happen to first found `IntegerSetting` or `BooleanSetting` that
    /// matches with the argument `new_value`.
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

    /// Save settings to a file specified by const `SETTINGS_FILE_NAME`.
    ///
    /// Saves current settings from `Vec<SettingsContainer>` field and game controller
    /// mappings from `Vec<String>`.
    ///
    /// For file format example, see load function's documentation.
    ///
    /// If saving the file fails, error message will be printed to
    /// standard output.
    pub fn save(&self) {
        let mut settings_text = String::new();

        settings_text.push_str("# Settings file for Space Boss Battles\n\n[Settings]\n");

        for setting in &self.settings {
            match setting.get_value() {
                SettingType::Boolean(_, value) => {
                    settings_text.push_str(setting.get_name());
                    settings_text.push('=');
                    settings_text.push_str(&value.to_string());
                    settings_text.push('\n');
                },
                SettingType::Integer(_, value) => {
                    settings_text.push_str(setting.get_name());
                    settings_text.push('=');
                    settings_text.push_str(&value.to_string());
                    settings_text.push('\n');
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

    /// Load settings from a file specified by const `SETTINGS_FILE_NAME`.
    ///
    /// If opening or reading the settings file fails or there is parsing error, an error message
    /// will be printed out to standard output.
    ///
    /// # File format
    ///
    /// Note that parser will trim every line it reads from the file.
    ///
    /// Empty lines will be skipped and lines starting with `#` will be treated as comments.
    ///
    /// If parser finds `[Settings]` section, it tries to parse key-value pairs `setting name=value` and
    /// match that key-value pair to available settings in `Vec<SettingsContainer>` field.
    ///
    /// If parser finds `[GameControllerMappings]` section, it adds all following non empty lines to
    /// `Vec<String>` field named `controller_mappings`.
    ///
    /// ## Example file
    ///
    /// ```text
    /// # Settings file for Space Boss Battles
    ///
    /// [Settings]
    /// Full screen=false
    /// FPS counter=false
    /// VSync=true
    /// Music volume=128
    /// Effect volume=128
    ///
    /// [GameControllerMappings]
    /// # https://wiki.libsdl.org/SDL_GameControllerAddMapping
    ///
    /// # In the generated documentation, the following game controller mapping
    /// # may be wrapped to multiple lines but its really a one line of text.
    /// 03000000100800000300000010010000,USB Gamepad , a:b2, b:b1, y:b0, x:b3, start:b9, guide:b12, back:b8, dpup:h0.1, dpleft:h0.8, dpdown:h0.4, dpright:h0.2, leftshoulder:b6, rightshoulder:b7, leftstick:b10, rightstick:b11, leftx:a0, lefty:a1, rightx:a3, righty:a2, lefttrigger:b4, righttrigger:b5
    ///
    /// ```
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

        let mut settings_parser = None;

        for line in settings_text.lines() {
            let line = line.trim();

            if line == "" || line.starts_with("#") {
                continue;
            } else if line == "[Settings]" {
                settings_parser = Some(SettingsParserMode::Settings);
                continue;
            } else if line == "[GameControllerMappings]" {
                settings_parser = Some(SettingsParserMode::GameControllerMappings);
                continue;
            }

            match settings_parser {
                Some(SettingsParserMode::Settings) => {
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
                Some(SettingsParserMode::GameControllerMappings) => {
                    self.controller_mappings.push(line.to_string());
                },
                None => (),
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

    /// Adds game controller mapping to `Vec<String>` located at `controller_mappings` field.
    pub fn add_game_controller_mapping(&mut self, mapping: String) {
        self.controller_mappings.push(mapping);
    }

    /// Is joystick event printing enabled.
    pub fn print_joystick_events(&self) -> bool {
        self.command_line_arguments.print_joystick_events
    }

    /// Is fps count printing enabled.
    pub fn print_fps_count(&self) -> bool {
        self.command_line_arguments.print_fps_count
    }

    /// Applies current settings from field `settings`.
    pub fn apply_current_settings<T: Renderer>(&self, renderer: &mut T, gui: &mut GUI, audio_manager: &mut AudioManager) {
        for setting in &self.settings {
            Settings::apply_setting(setting.get_value(), renderer, gui, audio_manager);
        }
    }

    /// Apply setting provided as argument.
    pub fn apply_setting<T: Renderer>(setting: SettingType, renderer: &mut T, gui: &mut GUI, audio_manager: &mut AudioManager) {
        match setting {
            SettingType::Boolean(BooleanSetting::FullScreen, value) => renderer.full_screen(value),
            SettingType::Boolean(BooleanSetting::ShowFpsCounter, value) => gui.set_show_fps_counter(value),
            SettingType::Boolean(BooleanSetting::VSync , value)  => renderer.v_sync(value),
            SettingType::Integer(IntegerSetting::SoundEffectVolume, value) => audio_manager.set_sound_effect_volume(Volume::new(value)),
            SettingType::Integer(IntegerSetting::MusicVolume, value) => audio_manager.set_music_volume(Volume::new(value)),
        }
    }
}

/// Settings parser states.
enum SettingsParserMode {
    Settings,
    GameControllerMappings,
}


/// Setting and it's name as text.
pub struct SettingContainer {
    name: &'static str,
    setting_type: SettingType,
}

impl SettingContainer {
    /// Create new `SettingContainer`.
    pub fn new(name: &'static str, setting_type: SettingType) -> SettingContainer {
        SettingContainer { name, setting_type }
    }

    /// Try setting a new boolean value to the `SettingContainer`.
    ///
    /// Returns true if new value was set.
    ///
    /// Works only if `SettingsContainer`'s current setting is same `BooleanSetting` as argument `setting`.
    fn set_if_boolean_setting_matches(&mut self, setting: BooleanSetting, value: bool) -> bool {
        if let &mut SettingType::Boolean(container_setting, ref mut old_value) = &mut self.setting_type {
            if container_setting == setting {
                *old_value = value;
                return true;
            }
        }

        false
    }

    /// Try setting a new integer value to the `SettingContainer`.
    ///
    /// Returns true if new value was set.
    ///
    /// Works only if `SettingsContainer`'s current setting is same `IntegerSetting` as argument `setting`.
    fn set_if_integer_setting_matches(&mut self, setting: IntegerSetting, value: i32) -> bool {
        if let &mut SettingType::Integer(container_setting, ref mut old_value) = &mut self.setting_type {
            if container_setting == setting {
                *old_value = value;
                return true;
            }
        }

        false
    }

    /// Get setting's name text.
    pub fn get_name(&self) -> &str {
        self.name
    }

    /// Get setting data.
    pub fn get_value(&self) -> SettingType {
        self.setting_type
    }
}

/// Parsed command line arguments.
///
/// # Supported arguments
/// * `--fps`
/// * `--joystick-events`
/// * `--help` or `-h`
/// * `--music path_to_music_file`
pub struct Arguments {
    show_help: bool,
    print_fps_count: bool,
    print_joystick_events: bool,
    music_file_path: Option<String>,
}

impl Arguments {
    /// Parse command line arguments
    ///
    /// Returns with Err(unknown_argument) if there is
    /// unknown argument.
    pub fn parse(args: Args) -> Result<Arguments, String> {
        let mut arguments = Arguments {
            show_help: false,
            print_fps_count: false,
            print_joystick_events: false,
            music_file_path: None,
        };

        let mut argument_parser_state = None;

        for arg in args.skip(1) {
            match argument_parser_state {
                Some(ArgumentParserState::MusicFilePath) => {
                    arguments.music_file_path = Some(arg);
                    argument_parser_state = None;
                },
                None => {
                    if arg == "--fps" {
                        arguments.print_fps_count = true;
                    } else if arg == "--joystick-events" {
                        arguments.print_joystick_events = true;
                    } else if arg == "--help" || arg == "-h" {
                        arguments.show_help = true;
                    } else if arg == "--music" {
                        argument_parser_state = Some(ArgumentParserState::MusicFilePath);
                    } else {
                        return Err(arg);
                    }
                },
            }
        }

        // TODO: Return error if argument_parser_state is not None
        //       at the end of argument parsing.

        Ok(arguments)
    }

    /// Is there argument `--help` or `-h` found.
    pub fn show_help(&self) -> bool {
        self.show_help
    }

    /// Possible user defined music file path.
    pub fn music_file_path(&self) -> &Option<String> {
        &self.music_file_path
    }
}

/// State for parsing the next argument.
enum ArgumentParserState {
    MusicFilePath,
}