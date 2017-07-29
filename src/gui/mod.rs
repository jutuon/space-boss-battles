/*
src/gui/mod.rs, 2017-07-29

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

pub mod components;

use gui::components::*;

use input::Input;

#[derive(Copy, Clone)]
pub enum GUIEvent {
    ChangeState(GUIState),
    Exit,
}

#[derive(Copy, Clone)]
pub enum GUIState {
    MainMenu,
    PauseMenu,
    Game,
}


pub trait GUILayer {
    fn components(&self) -> &[GUIButton];
    fn handle_event<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent>;
}


pub struct GUI {
    main_menu: MainMenu,
    pause_menu: PauseMenu,
    state: GUIState,
    render_game: bool,
    update_game: bool,
}


impl GUI {
    pub fn new() -> GUI {
        GUI {
            main_menu: MainMenu::new(),
            pause_menu: PauseMenu::new(),
            state: GUIState::MainMenu,
            render_game: false,
            update_game: false,
        }
    }

    pub fn get_state(&self) -> &GUIState {
        &self.state
    }

    pub fn render_game(&self) -> bool {
        self.render_game
    }

    pub fn update_game(&self) -> bool {
        self.update_game
    }
}

impl GUILayer for GUI {
    fn components(&self) -> &[GUIButton] {
        match self.state {
            GUIState::MainMenu => self.main_menu.components(),
            GUIState::PauseMenu => self.pause_menu.components(),
            _ => &[],
        }
    }

    fn handle_event<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
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
            _ => (),
        };

        event
    }
}


pub struct MainMenu {
     buttons: GUIGroup<GUIButton>,
}


impl MainMenu {
    fn new() -> MainMenu {
        let width = 5.0;
        let height = 1.0;

        let buttons = GUIGroup::new(GUIButton::new(0.0, 1.0, width, height))
            .add(GUIButton::new(0.0, -1.0, width, height))
            .add(GUIButton::new(0.0, -3.0, width, height));

        MainMenu {buttons}
    }
}

impl GUILayer for MainMenu {
    fn components(&self) -> &[GUIButton] {
        self.buttons.get_components()
    }

    fn handle_event<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {
        if input.key_hit_up() {
            self.buttons.selection_up();
            None
        } else if input.key_hit_down() {
            self.buttons.selection_down();
            None
        } else if input.key_hit_enter() {
            let i = self.buttons.get_selection_index();
            match i {
                0 => Some(GUIEvent::ChangeState(GUIState::Game)),
                2 => Some(GUIEvent::Exit),
                _ => None,
            }
        } else {
            None
        }
    }
}


pub struct PauseMenu {
    buttons: GUIGroup<GUIButton>,
}

impl PauseMenu {
    fn new() -> PauseMenu {
        let width = 5.0;
        let height = 1.0;

        let buttons = GUIGroup::new(GUIButton::new(0.0, 1.0, width, height))
            .add(GUIButton::new(0.0, -1.0, width, height));

        PauseMenu {buttons}
    }

}

impl GUILayer for PauseMenu {
    fn components(&self) -> &[GUIButton] {
        self.buttons.get_components()
    }

    fn handle_event<T: Input>(&mut self, input: &mut T) -> Option<GUIEvent> {

        if input.key_hit_up() {
            self.buttons.selection_up();
            None
        } else if input.key_hit_down() {
            self.buttons.selection_down();
            None
        } else if input.key_hit_enter() {
            let i = self.buttons.get_selection_index();
            match i {
                0 => Some(GUIEvent::ChangeState(GUIState::Game)),
                1 => {
                    self.buttons.selection_up();
                    Some(GUIEvent::ChangeState(GUIState::MainMenu))
                },
                _ => None,
            }
        } else {
            None
        }
    }
}