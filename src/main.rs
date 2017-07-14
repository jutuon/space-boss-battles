/*
src/main.rs, 2017-07-14

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

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use renderer::Renderer;
use logic::Logic;

fn main() {
    println!("Hello, world!");

    let sdl_context = sdl2::init().expect("sdl2 init failed");
    let mut event_pump = sdl_context.event_pump().expect("failed to get handle to sdl2 event_pump");

    let video = sdl_context.video().expect("video subsystem init fail");

    let mut renderer = renderer::OpenGLRenderer::new(video);
    let mut game = Game::new();

    'main: loop {
        for event in event_pump.poll_iter() {
            if game.handle_event(event) {
                break 'main
            }
        }

        game.update();

        game.render(&mut renderer);
    }

}

pub struct Game {
    game_logic: Logic,
}

impl Game {
    pub fn new() -> Game {
        let game_logic = Logic::new();
        Game {game_logic}
    }

    pub fn handle_event(&self, event: Event) -> bool {
        match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => true,
                _ => false,
        }
    }

    pub fn render<T: Renderer>(&self, renderer: &mut T) {
        renderer.start();
        renderer.render(&self.game_logic);
        renderer.end();
    }

    pub fn update(&mut self) {

    }
}