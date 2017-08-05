/*
src/gui/settings.rs, 2017-08-05

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use gui::GUIEvent;

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
}

impl Settings {
    pub fn new() -> Settings {
        let settings = vec![
            SettingContainer::new("Full screen", SettingType::Boolean(SettingEvent::FullScreen, false)),
            SettingContainer::new("FPS counter", SettingType::Boolean(SettingEvent::ShowFpsCounter, false)),
            SettingContainer::new("VSync", SettingType::Boolean(SettingEvent::VSync, true)),
        ];

        Settings {
            settings: settings,
        }
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
}

impl Setting for SettingContainer {
    fn get_name(&self) -> &str {
        self.name
    }

    fn get_value(&self) -> SettingType {
        self.setting_type
    }
}