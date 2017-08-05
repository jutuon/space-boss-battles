/*
src/gui/mod.rs, 2017-08-05

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

pub trait GUILayerComponents {
    fn components(&self) -> (&[GUIButton], &[GUIText]);
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

impl GUILayerComponents for GUI {
    fn components(&self) -> (&[GUIButton], &[GUIText]) {
        match self.state {
            GUIState::MainMenu => self.main_menu.components(),
            GUIState::PauseMenu => self.pause_menu.components(),
            _ => (&[], &[]),
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
    fn components(&self) -> (&[GUIButton], &[GUIText]) {
        (self.buttons.get_components(), &self.texts)
    }
}

impl GUIBasicLayer for MainMenu {
    fn get_buttons_mut(&mut self) -> &mut GUIGroup<GUIButton> { &mut self.buttons }

    fn event_from_index(&mut self, i: usize) -> Option<GUIEvent> {
        match i {
            0 => Some(GUIEvent::ChangeState(GUIState::Game)),
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
    fn components(&self) -> (&[GUIButton], &[GUIText]) {
        (self.buttons.get_components(), &self.texts)
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