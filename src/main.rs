/*
src/main.rs, 2017-07-15

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/


extern crate sdl2;
extern crate gl;
extern crate time;
extern crate image;
extern crate cgmath;


mod gui;
mod logic;
mod renderer;
mod input;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use renderer::Renderer;
use logic::Logic;

use input::{Input, InputKeyboard};

fn main() {
    let sdl_context = sdl2::init().expect("sdl2 init failed");
    let mut event_pump = sdl_context.event_pump().expect("failed to get handle to sdl2 event_pump");

    let video = sdl_context.video().expect("video subsystem init fail");

    let mut renderer = renderer::OpenGLRenderer::new(video);
    let mut game = Game::new();

    loop {
        if game.quit() {
            break;
        }

        for event in event_pump.poll_iter() {
            game.handle_event(event);
        }

        game.update();

        game.render(&mut renderer);
    }

}

pub struct Game {
    game_logic: Logic,
    quit: bool,
    input: InputKeyboard,
}

impl Game {
    pub fn new() -> Game {
        let game_logic = Logic::new();
        let quit = false;
        let input = InputKeyboard::new();
        Game {game_logic, quit, input}
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => self.quit = true,
                Event::KeyDown {keycode: Some(key), ..} => self.input.update_key_down(key),
                Event::KeyUp {keycode: Some(key), ..} => self.input.update_key_up(key),
                _ => (),
        }
    }

    pub fn render<T: Renderer>(&self, renderer: &mut T) {
        renderer.start();
        renderer.render(&self.game_logic);
        renderer.end();
    }

    pub fn update(&mut self) {
        self.game_logic.update(&self.input);
    }
}