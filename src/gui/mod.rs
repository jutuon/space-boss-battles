/*
src/gui/mod.rs, 2017-08-09

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

pub mod components;

const BUTTON_WIDTH: f32 = 5.0;
const BUTTON_HEIGHT: f32 = 1.0;


use gui::components::*;

use input::Input;
use logic::{Difficulty, MovingBackground};
use settings::{ Settings, SettingType, Setting};


#[derive(Copy, Clone)]
pub enum GUIEvent {
    NextLevel,
    NewGame(Difficulty),
    ChangeState(GUIState),
    ChangeSetting(SettingType),
    Exit,
}

#[derive(Copy, Clone)]
pub enum GUIState {
    MainMenu,
    DifficultySelectionMenu,
    PauseMenu,
    Game,
    PlayerWinsScreen,
    NextLevelScreen,
    GameOverScreen,
    SettingsMenu,
}


pub trait GUILayer {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton<GUIEvent>>;

    fn layer_specific_operations(&mut self, event: GUIEvent) {}
}

pub trait GUILayerEventHandler
    where Self: GUILayer {

    fn handle_event<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
        if input.key_hit_up() {
            self.get_buttons_mut().selection_up();
            None
        } else if input.key_hit_down() {
            self.get_buttons_mut().selection_down();
            None
        } else if input.key_hit_enter() {
            let event = self.get_buttons_mut().action_of_currently_selected_component();
            self.layer_specific_operations(event);
            Some(event)
        } else if input.mouse_button_hit() {
            let option_event = self.get_buttons_mut().check_collision_and_return_action(input.mouse_location());

            if let Some(event) = option_event {
                self.layer_specific_operations(event);
            }

            option_event
        } else if input.mouse_motion() {
            self.get_buttons_mut().update_selection(input.mouse_location());
            None
        } else {
            None
        }
    }
}

pub struct GUIComponentReferences<'a> {
    buttons: &'a [GUIButton<GUIEvent>],
    texts: &'a [GUIText],
    health_bars: &'a [GUIHealthBar],
}

impl <'a> GUIComponentReferences<'a> {
    fn new() -> GUIComponentReferences<'a> {
        GUIComponentReferences {
            buttons: &[],
            texts: &[],
            health_bars: &[],
        }
    }

    fn set_buttons(mut self, buttons: &'a [GUIButton<GUIEvent>]) -> GUIComponentReferences<'a> {
        self.buttons = buttons;
        self
    }

    fn set_texts(mut self, texts: &'a [GUIText]) -> GUIComponentReferences<'a> {
        self.texts = texts;
        self
    }

    fn set_health_bars(mut self, health_bars: &'a [GUIHealthBar]) -> GUIComponentReferences<'a> {
        self.health_bars = health_bars;
        self
    }

    pub fn buttons(&self) -> &[GUIButton<GUIEvent>] {
        self.buttons
    }

    pub fn texts(&self) -> &[GUIText] {
        self.texts
    }

    pub fn health_bars(&self) -> &[GUIHealthBar] {
        self.health_bars
    }
}

pub trait GUILayerComponents {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a>;
}


pub struct GUI {
    main_menu: BasicGUILayer,
    pause_menu: PauseMenu,
    settings_menu: SettingsMenu,
    game_status: GameStatus,
    difficulty_selection_menu: BasicGUILayer,
    state: GUIState,
    render_game: bool,
    update_game: bool,
    fps_counter: GUIFpsCounter,
    background: MovingBackground,
    game_over_screen: BasicGUILayer,
    player_wins_screen: BasicGUILayer,
    next_level_screen: BasicGUILayer,
}


impl GUI {
    pub fn new(settings: &Settings) -> GUI {
        let mut background = MovingBackground::new();
        background.move_position_x(0.25);

        GUI {
            main_menu: BasicGUILayer::main_menu(),
            pause_menu: PauseMenu::new(),
            settings_menu: SettingsMenu::new(settings),
            game_status: GameStatus::new(),
            difficulty_selection_menu: BasicGUILayer::difficulty_selection_menu(),
            state: GUIState::MainMenu,
            render_game: false,
            update_game: false,
            fps_counter: GUIFpsCounter::new(-5.0, 3.2),
            background,
            game_over_screen: BasicGUILayer::game_over_screen(),
            player_wins_screen: BasicGUILayer::player_wins_screen(),
            next_level_screen: BasicGUILayer::next_level_screen(),
        }
    }

    pub fn render_game(&self) -> bool {
        self.render_game
    }

    pub fn update_game(&self) -> bool {
        self.update_game
    }

    pub fn update_settings(&mut self, settings: &Settings) {
        self.settings_menu.update_settings_status_texts(settings);
    }

    pub fn handle_event<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
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
            GUIState::SettingsMenu => self.settings_menu.handle_event(input),
            GUIState::DifficultySelectionMenu => self.difficulty_selection_menu.handle_event(input),
            GUIState::NextLevelScreen => self.next_level_screen.handle_event(input),
            GUIState::GameOverScreen => self.game_over_screen.handle_event(input),
            GUIState::PlayerWinsScreen => self.player_wins_screen.handle_event(input),

        };

        if let Some(event) = event {
            self.handle_gui_event(event);
        }

        event
    }

    pub fn handle_gui_event(&mut self, event: GUIEvent ) {
        match event {
            GUIEvent::ChangeState(GUIState::Game) | GUIEvent::NextLevel => {
                self.render_game = true;
                self.update_game = true;
                self.state = GUIState::Game;
            },
            GUIEvent::ChangeState(state @ GUIState::PauseMenu) |
            GUIEvent::ChangeState(state @ GUIState::GameOverScreen) |
            GUIEvent::ChangeState(state @ GUIState::NextLevelScreen) |
            GUIEvent::ChangeState(state @ GUIState::PlayerWinsScreen) => {
                self.render_game = true;
                self.update_game = false;
                self.state = state;
            },
            GUIEvent::ChangeState(state @ GUIState::MainMenu) => {
                self.render_game = false;
                self.update_game = false;
                self.state = state;
            },
            GUIEvent::ChangeState(state @ GUIState::DifficultySelectionMenu) => {
                self.render_game = false;
                self.update_game = false;
                self.state = state;
            },
            GUIEvent::ChangeState(state @ GUIState::SettingsMenu) => {
                self.render_game = false;
                self.update_game = false;
                self.state = state;
            },
            GUIEvent::NewGame(_) => {
                self.render_game = true;
                self.update_game = true;
                self.state = GUIState::Game;
            }
            _ => (),
        };
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

    pub fn get_game_status(&mut self) -> &mut GameStatus {
        &mut self.game_status
    }

    pub fn get_background(&self) -> &MovingBackground {
        &self.background
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
            GUIState::DifficultySelectionMenu => self.difficulty_selection_menu.components(),
            GUIState::GameOverScreen => self.game_over_screen.components(),
            GUIState::PlayerWinsScreen => self.player_wins_screen.components(),
            GUIState::NextLevelScreen => self.next_level_screen.components(),
        }
    }
}





pub struct BasicGUILayer {
     buttons: GUIGroup<GUIButton<GUIEvent>>,
     texts: Vec<GUIText>,
}

impl BasicGUILayer {
    fn main_menu() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Start Game", GUIEvent::ChangeState(GUIState::DifficultySelectionMenu)))
                              .add(GUIButton::new(0.0, -1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Settings", GUIEvent::ChangeState(GUIState::SettingsMenu)))
                              .add(GUIButton::new(0.0, -3.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Exit", GUIEvent::Exit)),
            texts: vec![GUIText::new(0.0, 3.0, "Space Boss Battles")],
        }
    }

    fn difficulty_selection_menu() -> BasicGUILayer {
        let mut buttons = GUIGroup::new(GUIButton::new(0.0, 1.5, BUTTON_WIDTH, BUTTON_HEIGHT, "Easy", GUIEvent::NewGame(Difficulty::Easy)))
            .add(GUIButton::new(0.0, 0.2, BUTTON_WIDTH, BUTTON_HEIGHT, "Normal", GUIEvent::NewGame(Difficulty::Normal)))
            .add(GUIButton::new(0.0, -1.1, BUTTON_WIDTH, BUTTON_HEIGHT, "Hard", GUIEvent::NewGame(Difficulty::Hard)))
            .add(GUIButton::new(0.0, -2.7, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu)));

        buttons.selection_down();

        BasicGUILayer {
            buttons,
            texts: vec![GUIText::new(0.0, 3.0, "Select game difficulty")],
        }
    }

    fn player_wins_screen() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu))),
            texts: vec![GUIText::new(0.0, 3.0, "Congratulations, you won the game")],
        }
    }

    fn game_over_screen() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu))),
            texts: vec![GUIText::new(0.0, 3.0, "Game Over")],
        }
    }

    fn next_level_screen() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Next Level", GUIEvent::NextLevel)),
            texts: vec![GUIText::new(0.0, 3.0, "Congratulations, you won")],
        }
    }
}

impl GUILayerComponents for BasicGUILayer {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_buttons(self.buttons.get_components()).set_texts(&self.texts)
    }
}

impl GUILayer for BasicGUILayer {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton<GUIEvent>> { &mut self.buttons }
}

impl GUILayerEventHandler for BasicGUILayer {}


pub struct PauseMenu(BasicGUILayer);

impl PauseMenu {
    fn new() -> PauseMenu {
        PauseMenu(
            BasicGUILayer {
                buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Continue", GUIEvent::ChangeState(GUIState::Game)))
                                .add(GUIButton::new(0.0, -1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu))),
                texts: vec![GUIText::new(0.0, 3.0, "Game Paused")],
            }
        )
    }
}

impl GUILayerComponents for PauseMenu {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> { self.0.components() }
}

impl GUILayer for PauseMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton<GUIEvent>> { self.0.get_buttons_mut() }

    fn layer_specific_operations(&mut self, event: GUIEvent) {
        if let GUIEvent::ChangeState(GUIState::MainMenu) = event {
            self.0.buttons.selection_up();
        }
    }
}

impl GUILayerEventHandler for PauseMenu {}


pub struct GameStatus {
    health_bars: [GUIHealthBar; 2],
}

impl GameStatus {
    fn new() -> GameStatus {
        GameStatus {
            health_bars: [
                GUIHealthBar::new(GUIComponentAlignment::Left, 4.0),
                GUIHealthBar::new(GUIComponentAlignment::Right,4.0),
            ],
        }
    }

    pub fn set_player_health(&mut self, health: u32) {
        self.health_bars[0].update_health(health);
    }

    pub fn set_enemy_health(&mut self, health: u32) {
        self.health_bars[1].update_health(health);
    }
}

impl GUIUpdatePosition for GameStatus {
    fn update_position_from_half_screen_width(&mut self, width: f32) {
        self.health_bars[0].update_position_from_half_screen_width(width);
        self.health_bars[1].update_position_from_half_screen_width(width);
    }
}

impl GUILayerComponents for GameStatus {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_health_bars(&self.health_bars)
    }
}


pub struct SettingsMenu(BasicGUILayer);

impl SettingsMenu {
    fn new(settings: &Settings) -> SettingsMenu {
        let x_button = -2.0;
        let x_text = 3.0;
        let mut y = 1.5;

        let mut gui_group_builder = GUIGroupBuilder::new();
        let mut texts = Vec::new();

        for setting in settings.get_settings() {
            gui_group_builder.add(GUIButton::new(x_button, y, BUTTON_WIDTH, BUTTON_HEIGHT, setting.get_name(), GUIEvent::ChangeSetting(setting.get_value())));

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
        let buttons = buttons.add(GUIButton::new(x_button, y, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu)));

        SettingsMenu(BasicGUILayer {buttons, texts})
    }

    fn update_settings_status_texts(&mut self, settings: &Settings) {
        for (setting, text) in settings.get_settings().iter().zip(self.0.texts.iter_mut()) {

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
        self.0.components()
    }
}

impl GUILayer for SettingsMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton<GUIEvent>> { self.0.get_buttons_mut() }
}

impl GUILayerEventHandler for SettingsMenu {}
