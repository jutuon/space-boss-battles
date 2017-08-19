/*
src/gui/mod.rs, 2017-08-17

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Simple GUI toolkit with events.
//!
//! # Implementation
//!
//! `GUI` type stores several types that implements `GUILayer` trait.
//! `GUILayer` is layer which consists of several GUI components like
//! like `GUIButton` or `GUIText`.
//!
//! `GUI` stores info about currently active `GUILayer`.
//! There can only be one active `GUILayer` at a time.
//!
//! `GUI` will call `handle_input` method
//! of currently active `GUILayer` when input should be updated.
//! `GUILayer` will check the if there is adequate input for
//! updating it's state, like changing `GUIButton`'s color when its selected, and
//! will possibly send an `GUIEvent` back to the `GUI`.
//!
//! `GUIEvent` type represents request of change to the current state of `GUI` or
//! some other component of the game. That means you can request starting a new game or
//! changing some setting also.

pub mod components;

const BUTTON_WIDTH: f32 = 5.0;
const BUTTON_HEIGHT: f32 = 1.0;

const FPS_COUNTER_POSITION_Y: f32 = 3.2;

use gui::components::*;

use input::Input;
use logic::Difficulty;
use settings::{ Settings, SettingType, BooleanSetting, IntegerSetting};

use audio;


/// Event that will be sent from `GUILayer` to `GUI`.
#[derive(Copy, Clone)]
pub enum GUIEvent {
    NextLevel,
    NewGame(Difficulty),
    ChangeState(GUIState),
    ChangeSetting(SettingType),
    Exit,
}

/// Current state of the GUI.
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

/// Component information for rendering is only required for GUILayer.
pub trait GUILayer {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a>;
}

/// Input handling for GUILayer.
///
/// Includes default implementation for handling input for vertical button groups.
pub trait GUILayerInputHandler : GUILayer {
    /// Implementation for this is required for default input handling.
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton>;

    /// Override this method to do something before sending the `GUIEvent`
    /// to the `GUI`.
    fn layer_specific_operations(&mut self, _event: &mut GUIEvent) {}

    /// Override this method to do additional input handling in addition
    /// to the default input handling. This method will be called
    /// in the else block of default `handle_input` function.
    fn layer_specific_input_handling<T: Input>(&mut self, _input: &mut T) -> Option<GUIEvent> { None }

    /// Default implementation for handling input for vertical button groups.
    /// Keyboard and mouse input are supported.
    fn handle_input<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
        if input.key_hit_up() {
            self.get_buttons_mut().selection_up();
            None
        } else if input.key_hit_down() {
            self.get_buttons_mut().selection_down();
            None
        } else if input.key_hit_enter() {
            let mut event = self.get_buttons_mut().event_of_currently_selected_component();
            self.layer_specific_operations(&mut event);
            Some(event)
        } else if input.mouse_button_hit() {
            let mut option_event = self.get_buttons_mut().check_collision_and_return_event(input.mouse_location());

            if let &mut Some(ref mut event) = &mut option_event {
                self.layer_specific_operations(event);
            }

            option_event
        } else if input.mouse_motion() {
            self.get_buttons_mut().update_selection(input.mouse_location());
            None
        } else {
            self.layer_specific_input_handling(input)
        }
    }
}

/// Slices of different types of GUI components.
/// Currently used only for rendering the components.
pub struct GUIComponentReferences<'a> {
    buttons: &'a [GUIButton],
    texts: &'a [GUIText],
    health_bars: &'a [GUIHealthBar],
}

impl <'a> GUIComponentReferences<'a> {
    /// Create new GUIComponentReferences with empty slices.
    fn new() -> GUIComponentReferences<'a> {
        GUIComponentReferences {
            buttons: &[],
            texts: &[],
            health_bars: &[],
        }
    }

    /// Set `GUIButton` slice.
    fn set_buttons(mut self, buttons: &'a [GUIButton]) -> GUIComponentReferences<'a> {
        self.buttons = buttons;
        self
    }

    /// Set `GUIText` slice.
    fn set_texts(mut self, texts: &'a [GUIText]) -> GUIComponentReferences<'a> {
        self.texts = texts;
        self
    }

    /// Set `GUIHealthBar` slice.
    fn set_health_bars(mut self, health_bars: &'a [GUIHealthBar]) -> GUIComponentReferences<'a> {
        self.health_bars = health_bars;
        self
    }

    /// Get `GUIButton` slice.
    pub fn buttons(&self) -> &[GUIButton] {
        self.buttons
    }

    /// Get `GUIText` slice.
    pub fn texts(&self) -> &[GUIText] {
        self.texts
    }

    /// Get `GUIHealthBar` slice.
    pub fn health_bars(&self) -> &[GUIHealthBar] {
        self.health_bars
    }
}


/// Stores GUI components and state.
pub struct GUI {
    main_menu: BasicGUILayer,
    pause_menu: PauseMenu,
    settings_menu: SettingsMenu,
    game_status: GameStatus,
    difficulty_selection_menu: BasicGUILayer,
    state: GUIState,
    fps_counter: GUIFpsCounter,
    game_over_screen: BasicGUILayer,
    player_wins_screen: BasicGUILayer,
    next_level_screen: BasicGUILayer,
}


impl GUI {
    /// Create new `GUI`.
    pub fn new(settings: &Settings) -> GUI {
        GUI {
            main_menu: BasicGUILayer::main_menu(),
            pause_menu: PauseMenu::new(),
            settings_menu: SettingsMenu::new(settings),
            game_status: GameStatus::new(),
            difficulty_selection_menu: BasicGUILayer::difficulty_selection_menu(),
            state: GUIState::MainMenu,
            fps_counter: GUIFpsCounter::new(0.0, FPS_COUNTER_POSITION_Y),
            game_over_screen: BasicGUILayer::game_over_screen(),
            player_wins_screen: BasicGUILayer::player_wins_screen(),
            next_level_screen: BasicGUILayer::next_level_screen(),
        }
    }

    /// Call `handle_input` function of current `GUILayer`.
    ///
    /// Updates `GUI`'s state according to `GUIEvent` returned by
    /// the current `GUILayer`.
    pub fn handle_input<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
        let event = match self.state {
            GUIState::MainMenu => self.main_menu.handle_input(input),
            GUIState::PauseMenu => {
                if input.key_hit_back() {
                    Some(GUIEvent::ChangeState(GUIState::Game))
                } else {
                    self.pause_menu.handle_input(input)
                }
            },
            GUIState::Game => {
                if input.key_hit_back() {
                    Some(GUIEvent::ChangeState(GUIState::PauseMenu))
                } else {
                    None
                }
            },
            GUIState::SettingsMenu => self.settings_menu.handle_input(input),
            GUIState::DifficultySelectionMenu => self.difficulty_selection_menu.handle_input(input),
            GUIState::NextLevelScreen => self.next_level_screen.handle_input(input),
            GUIState::GameOverScreen => self.game_over_screen.handle_input(input),
            GUIState::PlayerWinsScreen => self.player_wins_screen.handle_input(input),

        };

        if let Some(event) = event {
            self.handle_gui_event(event);
        }

        event
    }

    /// Update `GUI`'s state from `GUIEvent`.
    pub fn handle_gui_event(&mut self, event: GUIEvent ) {
        match event {
            GUIEvent::NextLevel | GUIEvent::NewGame(_) => self.state = GUIState::Game,
            GUIEvent::ChangeState(state) => self.state = state,
            _ => (),
        };
    }

    /// Update `GUIFpsCounter`.
    pub fn update_fps_counter(&mut self, count: u32) {
        self.fps_counter.update_fps_count(count);
    }

    /// Get `GUIFpsCounter`.
    pub fn get_gui_fps_counter(&self) -> &GUIFpsCounter {
        &self.fps_counter
    }

    /// Show or hide `GUIFpsCounter`.
    pub fn set_show_fps_counter(&mut self, value: bool) {
        self.fps_counter.set_show_fps(value);
    }

    /// Get `GUILayer` `GameStatus`.
    pub fn get_game_status(&mut self) -> &mut GameStatus {
        &mut self.game_status
    }

    /// Get current `GUILayer`'s components.
    pub fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
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

    /// Update positions of `GUIFpsCounter` and `GameStatus`.
    pub fn update_position_from_half_screen_width(&mut self, width: f32) {
        self.fps_counter.update_position_from_half_screen_width(width);
        self.game_status.update_position_from_half_screen_width(width);
    }
}

/// Base type for simple menus with `GUIButton`s and `GUIText`s.
pub struct BasicGUILayer {
     buttons: GUIGroup<GUIButton>,
     texts: Vec<GUIText>,
}

impl BasicGUILayer {
    /// Create main menu.
    fn main_menu() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Start Game", GUIEvent::ChangeState(GUIState::DifficultySelectionMenu)))
                              .add(GUIButton::new(0.0, -1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Settings", GUIEvent::ChangeState(GUIState::SettingsMenu)))
                              .add(GUIButton::new(0.0, -3.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Exit", GUIEvent::Exit)),
            texts: vec![GUIText::new(0.0, 3.0, "Space Boss Battles")],
        }
    }

    /// Create difficulty selection menu.
    fn difficulty_selection_menu() -> BasicGUILayer {
        let mut buttons = GUIGroup::new(GUIButton::new(0.0, 1.5, BUTTON_WIDTH, BUTTON_HEIGHT, "Easy", GUIEvent::NewGame(Difficulty::Easy)))
            .add(GUIButton::new(0.0, 0.2, BUTTON_WIDTH, BUTTON_HEIGHT, "Normal", GUIEvent::NewGame(Difficulty::Normal)))
            .add(GUIButton::new(0.0, -1.1, BUTTON_WIDTH, BUTTON_HEIGHT, "Hard", GUIEvent::NewGame(Difficulty::Hard)))
            .add(GUIButton::new(0.0, -2.7, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu)));

        // Set default selection to "Normal".
        buttons.selection_down();

        BasicGUILayer {
            buttons,
            texts: vec![GUIText::new(0.0, 3.0, "Select game difficulty")],
        }
    }

    /// Create player wins screen.
    fn player_wins_screen() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu))),
            texts: vec![GUIText::new(0.0, 3.0, "Congratulations, you won the game")],
        }
    }

    /// Create game over screen.
    fn game_over_screen() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu))),
            texts: vec![GUIText::new(0.0, 3.0, "Game Over")],
        }
    }

    /// Create next level screen.
    fn next_level_screen() -> BasicGUILayer {
        BasicGUILayer {
            buttons: GUIGroup::new(GUIButton::new(0.0, 1.0, BUTTON_WIDTH, BUTTON_HEIGHT, "Next Level", GUIEvent::NextLevel)),
            texts: vec![GUIText::new(0.0, 3.0, "Congratulations, you won")],
        }
    }
}

impl GUILayer for BasicGUILayer {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_buttons(self.buttons.get_components()).set_texts(&self.texts)
    }
}

impl GUILayerInputHandler for BasicGUILayer {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton> { &mut self.buttons }
}

/// New type `PauseMenu` because selected button
/// must be reset to "Continue" after pressing "Main Menu" button.
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

impl GUILayer for PauseMenu {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> { self.0.components() }
}

impl GUILayerInputHandler for PauseMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton> { self.0.get_buttons_mut() }

    /// Reset currently selected button to "Continue" after pressing "Main Menu" button.
    fn layer_specific_operations(&mut self, event: &mut GUIEvent) {
        if let &mut GUIEvent::ChangeState(GUIState::MainMenu) = event {
            self.0.buttons.selection_up();
        }
    }
}


/// New type `GameStatus` because game status
/// screen contains only two `GUIHealthBar`.
pub struct GameStatus {
    health_bars: [GUIHealthBar; 2],
}

impl GameStatus {
    /// Create new `GameStatus`.
    fn new() -> GameStatus {
        GameStatus {
            health_bars: [
                GUIHealthBar::new(GUIComponentAlignment::Left, 0.0, 4.0, 3.0, 100, 25, true),
                GUIHealthBar::new(GUIComponentAlignment::Right, 0.0, 4.0, 3.0, 100, 25, true),
            ],
        }
    }

    /// Updates players health bar.
    pub fn set_player_health(&mut self, health: u32) {
        self.health_bars[0].update_health(health);
    }

    /// Updates enemy health bar.
    pub fn set_enemy_health(&mut self, health: u32) {
        self.health_bars[1].update_health(health);
    }

    /// Update positions of `GUIHealthBar`s
    fn update_position_from_half_screen_width(&mut self, width: f32) {
        self.health_bars[0].update_position_from_half_screen_width(width);
        self.health_bars[1].update_position_from_half_screen_width(width);
    }
}

impl GUILayer for GameStatus {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        GUIComponentReferences::new().set_health_bars(&self.health_bars)
    }
}

// TODO: Audio volume sliders mouse support.

/// Create settings menu from `Settings`, create
/// updated setting values and send them with `GUIEvent`.
pub struct SettingsMenu {
    layer: BasicGUILayer,
    value_indicators: Vec<GUIHealthBar>,
}

impl SettingsMenu {
    /// Creates new settings menu from `Settings`.
    fn new(settings: &Settings) -> SettingsMenu {
        let x_button = -2.0;
        let x_text = 3.0;
        let mut y = 2.7;

        let mut gui_group_builder = GUIGroupBuilder::new();
        let mut texts = Vec::new();
        let mut value_indicators = Vec::new();

        for setting in settings.get_settings() {
            gui_group_builder.add(GUIButton::new(x_button, y, BUTTON_WIDTH, BUTTON_HEIGHT, setting.get_name(), GUIEvent::ChangeSetting(setting.get_value())));

            match setting.get_value() {
                SettingType::Boolean(_, true) => texts.push(GUIText::new(x_text, y, "Enabled")),
                SettingType::Boolean(_, false) => texts.push(GUIText::new(x_text, y, "Disabled")),
                SettingType::Integer(_, value) => {
                    let mut value_indicator = GUIHealthBar::new(GUIComponentAlignment::Center, x_text, y, 3.0, audio::MAX_VOLUME as u32, 0, false);
                    value_indicator.update_health(value as u32);
                    value_indicator.update_borders();
                    value_indicators.push(value_indicator);
                }
            }

            y -= 1.15;
        }

        texts.push(GUIText::new(0.0, 3.8, "Settings"));

        let buttons = gui_group_builder.create_gui_group();

        y -= 0.50;
        let buttons = buttons.add(GUIButton::new(x_button, y, BUTTON_WIDTH, BUTTON_HEIGHT, "Main Menu", GUIEvent::ChangeState(GUIState::MainMenu)));

        SettingsMenu {
            layer: BasicGUILayer {buttons, texts},
            value_indicators,
        }
    }

    /// Toggles boolean setting value if there exist an `GUIButton` with same values
    /// as arguments `setting` and `value`. Function will also update text related to that button.
    ///
    /// Returns the new setting value inside `SettingType` if matching button is found.
    fn update_boolean_setting(&mut self, setting: BooleanSetting, value: bool) -> Option<SettingType> {
        for (button, text) in self.layer.buttons.get_components_mut().iter_mut().zip(self.layer.texts.iter_mut()) {

            if let GUIEvent::ChangeSetting(SettingType::Boolean(events_boolean_setting, value2)) = button.event_data() {
                if setting == events_boolean_setting && value == value2 {

                    if value {
                        text.change_text("Disabled");
                    } else {
                        text.change_text("Enabled");
                    }
                    let new_setting = SettingType::Boolean(setting, !value);
                    button.set_event_data(GUIEvent::ChangeSetting(new_setting));
                    return Some(new_setting);
                }
            }

        }

        None
    }

    /// Tries adding argument `number` to current value of currently selected volume slider.
    ///
    /// To determine is volume slider selected, the method will check does the currently selected button
    /// contain an `IntegerSetting`. If button contains an `IntegerSetting`, the button's and slider's integer value will
    /// be updated and the new value will be returned as `GUIEvent`.
    fn update_currently_selected_integer_setting(&mut self, number: i32) -> Option<GUIEvent> {
         if let GUIEvent::ChangeSetting(SettingType::Integer(integer_setting, value)) = self.layer.buttons.event_of_currently_selected_component() {
            let value = audio::Volume::new(value + number).value();

            let updated_gui_event = GUIEvent::ChangeSetting(SettingType::Integer(integer_setting, value));
            self.layer.buttons.set_event_of_currently_selected_component(updated_gui_event);

            if let IntegerSetting::MusicVolume = integer_setting {
                self.value_indicators[0].update_health(value as u32);
            } else if let IntegerSetting::SoundEffectVolume = integer_setting {
                self.value_indicators[1].update_health(value as u32);
            }

            Some(updated_gui_event)
        } else {
            None
        }
    }
}

impl GUILayer for SettingsMenu {
    fn components<'a>(&'a self) -> GUIComponentReferences<'a> {
        self.layer.components().set_health_bars(&self.value_indicators)
    }
}

impl GUILayerInputHandler for SettingsMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton> { self.layer.get_buttons_mut() }

    /// Toggle `BooleanSetting`.
    fn layer_specific_operations(&mut self, event: &mut GUIEvent) {
        if let &mut GUIEvent::ChangeSetting(SettingType::Boolean(setting_event, value)) = event {
            if let Some(updated_setting) = self.update_boolean_setting(setting_event, value) {
                *event = GUIEvent::ChangeSetting(updated_setting);
            }
        }
    }

    /// Change slider values with left and right keys.
    fn layer_specific_input_handling<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
        if input.key_hit_left() {
            self.update_currently_selected_integer_setting(-20)
        } else if input.key_hit_right() {
            self.update_currently_selected_integer_setting(20)
        } else {
            None
        }
    }
}