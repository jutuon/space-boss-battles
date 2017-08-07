/*
src/gui/mod.rs, 2017-08-07

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

pub mod components;

use gui::components::*;

use input::Input;
use settings::{ Settings, SettingType, Setting};


#[derive(Copy, Clone)]
pub enum GUIEvent {
    ChangeState(GUIState),
    ChangeSetting(SettingType),
    SettingsUpdate(usize),
    Exit,
}

#[derive(Copy, Clone)]
pub enum GUIState {
    MainMenu,
    PauseMenu,
    Game,
    SettingsMenu,
}

pub struct GUIComponentReferences<'a> {
    buttons: &'a [GUIButton],
    texts: &'a [GUIText],
}

impl <'a> GUIComponentReferences<'a> {
    fn new() -> GUIComponentReferences<'a> {
        GUIComponentReferences {
            buttons: &[],
            texts: &[],
        }
    }

    fn set_buttons(mut self, buttons: &'a [GUIButton]) -> GUIComponentReferences<'a> {
        self.buttons = buttons;
        self
    }

    fn set_texts(mut self, texts: &'a [GUIText]) -> GUIComponentReferences<'a> {
        self.texts = texts;
        self
    }

    pub fn buttons(&self) -> &[GUIButton] {
        self.buttons
    }

    pub fn texts(&self) -> &[GUIText] {
        self.texts
    }
}

pub trait GUILayerComponents {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a>;
}


pub struct GUI {
    main_menu: MainMenu,
    pause_menu: PauseMenu,
    settings_menu: SettingsMenu,
    game_status: GameStatus,
    state: GUIState,
    render_game: bool,
    update_game: bool,
    fps_counter: GUIFpsCounter,
}


impl GUI {
    pub fn new(settings: &Settings) -> GUI {
        GUI {
            main_menu: MainMenu::new(),
            pause_menu: PauseMenu::new(),
            settings_menu: SettingsMenu::new(settings),
            game_status: GameStatus::new(),
            state: GUIState::MainMenu,
            render_game: false,
            update_game: false,
            fps_counter: GUIFpsCounter::new(-5.0, 3.0),
        }
    }

    pub fn render_game(&self) -> bool {
        self.render_game
    }

    pub fn update_game(&self) -> bool {
        self.update_game
    }

    pub fn handle_event<T: Input>(&mut self, input: &mut T, settings: &mut Settings) -> Option<GUIEvent> {
        let event = match self.state {
            GUIState::MainMenu => self.main_menu.handle_event(input),
            GUIState::PauseMenu => self.pause_menu.handle_event(input),
            GUIState::Game => {
                if input.key_hit_back() {
                    Some(GUIEvent::ChangeState(GUIState::PauseMenu))
                } else {
                    None
                }
            },
            GUIState::SettingsMenu => {
                let mut event = self.settings_menu.handle_event(input);
                if let Some(GUIEvent::SettingsUpdate(i)) = event {
                    event = settings.event_from_index(i);
                    self.settings_menu.update_settings_status_texts(settings);
                }

                event
            },
        };

        match event {
            None => (),
            Some(GUIEvent::ChangeState(state @ GUIState::Game)) => {
                self.render_game = true;
                self.update_game = true;
                self.state = state;
            },
            Some(GUIEvent::ChangeState(state @ GUIState::PauseMenu)) => {
                self.render_game = true;
                self.update_game = false;
                self.state = state;
            },
            Some(GUIEvent::ChangeState(state @ GUIState::MainMenu)) => {
                self.render_game = false;
                self.update_game = false;
                self.state = state;
            },
            Some(GUIEvent::ChangeState(state @ GUIState::SettingsMenu)) => {
                self.render_game = false;
                self.update_game = false;
                self.state = state;
            },
            _ => (),
        };

        event
    }

    pub fn update_fps_counter(&mut self, count: u32) {
        self.fps_counter.update_fps_count(count);
    }

    pub fn get_gui_fps_counter(&self) -> &GUIFpsCounter {
        &self.fps_counter
    }

    pub fn set_show_fps_counter(&mut self, value: bool) {
        self.fps_counter.set_show_fps(value);
    }
}

impl GUIUpdatePosition for GUI {
    fn update_position_from_half_screen_width(&mut self, width: f32) {
        self.fps_counter.update_position_from_half_screen_width(width);
        self.game_status.update_position_from_half_screen_width(width);
    }
}

impl GUILayerComponents for GUI {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        match self.state {
            GUIState::MainMenu => self.main_menu.components(),
            GUIState::PauseMenu => self.pause_menu.components(),
            GUIState::SettingsMenu => self.settings_menu.components(),
            GUIState::Game => self.game_status.components(),
        }
    }
}



pub trait GUIBasicLayer {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton>;

    fn event_from_index(&mut self, i: usize) -> Option<GUIEvent>;
}

pub trait GUILayerEventHandler
    where Self: GUIBasicLayer {

    fn handle_event<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
        if input.key_hit_up() {
            self.get_buttons_mut().selection_up();
            None
        } else if input.key_hit_down() {
            self.get_buttons_mut().selection_down();
            None
        } else if input.key_hit_enter() {
            let i = self.get_buttons_mut().get_selection_index();
            self.event_from_index(i)
        } else if input.mouse_button_hit() {
            match self.get_buttons_mut().get_collision_index(input.mouse_location()) {
                Some(i) => self.event_from_index(i),
                None => None,
            }
        } else if input.mouse_motion() {
            self.get_buttons_mut().update_selection(input.mouse_location());
            None
        } else {
            None
        }
    }
}



pub struct MainMenu {
     buttons: GUIGroup<GUIButton>,
     texts: [GUIText; 1],
}

impl MainMenu {
    fn new() -> MainMenu {
        let width = 5.0;
        let height = 1.0;

        let buttons = GUIGroup::new(GUIButton::new(0.0, 1.0, width, height, "Start Game"))
            .add(GUIButton::new(0.0, -1.0, width, height, "Settings"))
            .add(GUIButton::new(0.0, -3.0, width, height, "Exit"));

        let texts = [
            GUIText::new(0.0, 3.0, "Space Boss Battles"),
        ];

        MainMenu {buttons, texts}
    }

}

impl GUILayerComponents for MainMenu {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_buttons(self.buttons.get_components()).set_texts(&self.texts)
    }
}

impl GUIBasicLayer for MainMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton> { &mut self.buttons }

    fn event_from_index(&mut self, i: usize) -> Option<GUIEvent> {
        match i {
            0 => Some(GUIEvent::ChangeState(GUIState::Game)),
            1 => Some(GUIEvent::ChangeState(GUIState::SettingsMenu)),
            2 => Some(GUIEvent::Exit),
            _ => None,
        }
    }
}

impl GUILayerEventHandler for MainMenu {}


pub struct PauseMenu {
    buttons: GUIGroup<GUIButton>,
    texts: [GUIText; 1],
}

impl PauseMenu {
    fn new() -> PauseMenu {
        let width = 5.0;
        let height = 1.0;

        let buttons = GUIGroup::new(GUIButton::new(0.0, 1.0, width, height, "Continue"))
            .add(GUIButton::new(0.0, -1.0, width, height, "Main Menu"));

        let texts = [GUIText::new(0.0, 3.0, "Game Paused")];

        PauseMenu {buttons, texts}
    }
}

impl GUILayerComponents for PauseMenu {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_buttons(self.buttons.get_components()).set_texts(&self.texts)
    }
}

impl GUIBasicLayer for PauseMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton> { &mut self.buttons }

    fn event_from_index(&mut self, i: usize) -> Option<GUIEvent> {
        match i {
            0 => Some(GUIEvent::ChangeState(GUIState::Game)),
            1 => {
                self.buttons.selection_up();
                Some(GUIEvent::ChangeState(GUIState::MainMenu))
            },
            _ => None,
        }
    }
}

impl GUILayerEventHandler for PauseMenu {}


pub struct GameStatus {
    texts: [GUIText; 2],
}

impl GameStatus {
    fn new() -> GameStatus {
        let texts = [
            GUIText::new_with_alignment(-3.0, 4.0, "Player", GUIComponentAlignment::Left),
            GUIText::new_with_alignment(3.0, 4.0, "Enemy", GUIComponentAlignment::Right)
        ];

        GameStatus {texts}
    }
}

impl GUIUpdatePosition for GameStatus {
    fn update_position_from_half_screen_width(&mut self, width: f32) {
        self.texts[0].update_position_from_half_screen_width(width);
        self.texts[1].update_position_from_half_screen_width(width);
    }
}

impl GUILayerComponents for GameStatus {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_texts(&self.texts)
    }
}


pub struct SettingsMenu {
     buttons: GUIGroup<GUIButton>,
     texts: Vec<GUIText>,
}

impl SettingsMenu {
    fn new(settings: &Settings) -> SettingsMenu {
        let width = 5.0;
        let height = 1.0;

        let x_button = -2.0;
        let x_text = 3.0;
        let mut y = 1.5;

        let mut gui_group_builder = GUIGroupBuilder::new();
        let mut texts = Vec::new();

        for setting in settings.get_settings() {
            gui_group_builder.add(GUIButton::new(x_button, y, width, height, setting.get_name()));

            let text = match setting.get_value() {
                SettingType::Boolean(_, true) => "Enabled",
                SettingType::Boolean(_, false) => "Disabled",
            };

            texts.push(GUIText::new(x_text, y, text));

            y -= 1.25;
        }

        texts.push(GUIText::new(0.0, 3.0, "Settings"));

        let buttons = gui_group_builder.create_gui_group();

        y -= 0.50;
        let buttons = buttons.add(GUIButton::new(x_button, y, width, height, "Main Menu"));

        SettingsMenu {buttons, texts}
    }

    fn update_settings_status_texts(&mut self, settings: &Settings) {
        for (setting, text) in settings.get_settings().iter().zip(self.texts.iter_mut()) {

            let new_text = match setting.get_value() {
                SettingType::Boolean(_, true) => "Enabled",
                SettingType::Boolean(_, false) => "Disabled",
            };

            text.change_text(new_text);
        }
    }

}

impl GUILayerComponents for SettingsMenu {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_buttons(self.buttons.get_components()).set_texts(&self.texts)
    }
}

impl GUIBasicLayer for SettingsMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton> { &mut self.buttons }

    fn event_from_index(&mut self, i: usize) -> Option<GUIEvent> {
        if i == self.buttons.get_components().len() - 1 {
            Some(GUIEvent::ChangeState(GUIState::MainMenu))
        } else {
            Some(GUIEvent::SettingsUpdate(i))
        }
    }
}

impl GUILayerEventHandler for SettingsMenu {}
