/*
src/logic/mod.rs, 2017-08-10

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

pub mod common;

use logic::common::*;

use input::Input;

use time::PreciseTime;
use Timer;

use std::f32::consts;

use gui::{GUI, GUIState, GUIEvent};

use renderer::ModelMatrix;
use cgmath::Matrix4;

use rand::{Rng, ThreadRng};
use rand;

macro_rules! impl_model_matrix {
    ( $x:ty ) => {
        impl ModelMatrix for $x {
            fn model_matrix(&self) -> &Matrix4<f32> {
                &self.data().model_matrix
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

struct LogicSettings {
    screen_width_half: f32,
    player_laser_damage: i32,
    enemy_laser_damage: i32,
    enemy_hit_damage: i32,
}

impl LogicSettings {
    fn new() -> LogicSettings {
        LogicSettings {
            screen_width_half: 0.0,
            player_laser_damage: 0,
            enemy_laser_damage: 0,
            enemy_hit_damage: 0,
        }
    }

    fn settings_easy(&mut self) {
        self.player_laser_damage = 5;
        self.enemy_laser_damage = 1;
        self.enemy_hit_damage = 1;
    }

    fn settings_normal(&mut self) {
        self.player_laser_damage = 2;
        self.enemy_laser_damage = 5;
        self.enemy_hit_damage = 2;
    }

    fn settings_hard(&mut self) {
        self.player_laser_damage = 1;
        self.enemy_laser_damage = 10;
        self.enemy_hit_damage = 3;
    }
}

pub struct Logic {
    player: Player,
    enemy: Enemy,
    moving_background: MovingBackground,
    logic_settings: LogicSettings,
    level: u32,
    current_difficulty: Difficulty,
    game_running: bool,
    explosion: Explosion,
}

impl Logic {
    pub fn new() -> Logic {
        Logic {
            player: Player::new(),
            enemy: Enemy::new(),
            moving_background: MovingBackground::new(),
            logic_settings: LogicSettings::new(),
            level: 0,
            current_difficulty: Difficulty::Normal,
            game_running: true,
            explosion: Explosion::new(),
        }
    }

    pub fn update<T: Input>(&mut self, input: &T, gui: &mut GUI) {

        if self.game_running {
            self.player.update(input, &mut self.enemy, &self.logic_settings);
            self.enemy.update(&mut self.player, &self.logic_settings);
            self.moving_background.update();
        }

        self.explosion.update();

        if let Some(health) = self.player.health() {
            gui.get_game_status().set_player_health(health);

            if health == 0 {
                self.player.lasers.clear();
                self.enemy.lasers.clear();

                self.game_running = false;
                self.explosion.start_explosion(&self.player);
            }
        }

        if let Some(health) = self.enemy.health() {
            gui.get_game_status().set_enemy_health(health);

            if health == 0 {
                self.player.lasers.clear();
                self.enemy.lasers.clear();

                self.game_running = false;
                self.explosion.start_explosion(&self.enemy);
            }
        }

        if !self.game_running && self.explosion.explosion_finished() {
            if self.player.health == 0 {
                gui.handle_gui_event(GUIEvent::ChangeState(GUIState::GameOverScreen));
                self.player.visible = false;
            } else {
                if self.level == 1 {
                    gui.handle_gui_event(GUIEvent::ChangeState(GUIState::PlayerWinsScreen));
                } else {
                    gui.handle_gui_event(GUIEvent::ChangeState(GUIState::NextLevelScreen));
                }

                self.enemy.visible = false;
            }
        }
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }

    pub fn get_enemy(&self) -> &Enemy {
        &self.enemy
    }

    pub fn get_explosion(&self) -> &Explosion {
        &self.explosion
    }

    pub fn get_moving_background(&self) -> &MovingBackground {
        &self.moving_background
    }

    pub fn reset_game(&mut self, gui: &mut GUI, difficulty: Difficulty, level: u32) {
        if level > 3 {
            panic!("level index must be at range 0-3");
        }

        self.level = level;
        self.current_difficulty = difficulty;
        self.game_running = true;

        match difficulty {
            Difficulty::Easy => self.logic_settings.settings_easy(),
            Difficulty::Normal => self.logic_settings.settings_normal(),
            Difficulty::Hard => self.logic_settings.settings_hard(),
        }

        self.player.reset(&self.logic_settings);
        self.enemy.reset(&self.logic_settings, level);

        if let Some(health) = self.player.health() {
            gui.get_game_status().set_player_health(health);
        }

        if let Some(health) = self.enemy.health() {
            gui.get_game_status().set_enemy_health(health);
        }
    }

    pub fn reset_to_next_level(&mut self, gui: &mut GUI) {
        let difficulty = self.current_difficulty;
        let level = self.level + 1;
        self.reset_game(gui, difficulty, level);
    }

    pub fn update_half_screen_width(&mut self, half_width: f32) {
        self.logic_settings.screen_width_half = half_width;

    }
}

pub struct Particle {
    data: Data<f32>,
    speed: f32,
    lifetime_timer: Timer,
    lifetime_as_milliseconds: i64,
}

impl Particle {
    fn new(current_time: PreciseTime, x: f32, y: f32, angle: f32, speed: f32, lifetime_as_milliseconds: i64) -> Particle {

        let mut particle = Particle {
            data: Data::new(x, y, 0.1, 0.1),
            speed,
            lifetime_timer: Timer::new_from_time(current_time),
            lifetime_as_milliseconds,
        };
        particle.turn_without_updating_model_matrix(angle);

        particle
    }

    fn update(&mut self, current_time: PreciseTime) -> bool {
        let speed = self.speed;
        self.forward(speed);

        self.lifetime_timer.check(current_time, self.lifetime_as_milliseconds)
    }
}

impl GameObject for Particle {}
impl_model_matrix!(Particle);

impl GameObjectData<f32> for Particle {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}


pub struct Explosion {
    data: Data<f32>,
    visible: bool,
    timer: Timer,
    particles: Vec<Particle>,
    particle_creation_timer: Timer,
    rng: ThreadRng,
    remove_at_index: Vec<usize>,
}

impl Explosion {
    fn new() -> Explosion {
        Explosion {
            data: Data::new(0.0,0.0,0.5,0.5),
            visible: false,
            timer: Timer::new(),
            particles: Vec::new(),
            particle_creation_timer: Timer::new(),
            rng: rand::thread_rng(),
            remove_at_index: Vec::new(),
        }
    }

    pub fn start_explosion<T: GameObject>(&mut self, object: &T) {
        let current_time = PreciseTime::now();
        self.timer.reset(current_time);
        self.set_position(object.data().position.x, object.data().position.y);
        self.visible = true;
        self.particles.clear();
    }

    pub fn explosion_finished(&mut self) -> bool {
        if self.timer.check(PreciseTime::now(), 1500) {
            self.visible = false;
            true
        } else {
            false
        }
    }

    pub fn update(&mut self) {
        if !self.visible {
            return;
        }

        let current_time = PreciseTime::now();

        let mut remove_particles = false;
        for (i,particle) in self.particles.iter_mut().enumerate() {
            if particle.update(current_time) {
                self.remove_at_index.push(i);
                remove_particles = true;
            }
        }

        if remove_particles {
            for i in self.remove_at_index.iter().rev() {
                self.particles.remove(*i);
            }
            self.remove_at_index.clear();
        }

        if self.particle_creation_timer.check(current_time, 500) {
            let particle_count = 15;
            let circle_in_radians = consts::PI*2.0;
            for _ in 0..particle_count {
                let x = self.data().position.x;
                let y = self.data().position.y;
                self.particles.push(Particle::new(current_time, x, y, circle_in_radians * self.rng.gen::<f32>(), (self.rng.gen::<f32>()*0.02).max(0.01), (self.rng.gen::<u32>()%400+500) as i64));
            }
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn particles(&self) -> &Vec<Particle> {
        &self.particles
    }
}

impl GameObject for Explosion {}
impl_model_matrix!(Explosion);

impl GameObjectData<f32> for Explosion {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}

pub struct Player {
    data: Data<f32>,
    speed: f32,
    lasers: Vec<Laser>,
    laser_timer: Timer,
    health: i32,
    health_update: bool,
    visible: bool,
}

impl Player {
    fn new() -> Player {
        let data = Data::new(0.0, 0.0, 1.0, 1.0);
        let speed = 0.05;
        let lasers = vec![];
        let laser_timer = Timer::new();
        let health = 100;
        let health_update = true;
        Player { data, speed, lasers, laser_timer, health, health_update, visible: true, }
    }

    fn reset(&mut self, logic_settings: &LogicSettings) {
        self.data = Data::new(-3.0, 0.0, 1.0, 1.0);
        self.lasers.clear();
        self.health = 100;
        self.health_update = true;
        self.laser_timer.reset(PreciseTime::now());
        self.visible = true;
    }

    fn update(&mut self, input: &Input, enemy: &mut Enemy, logic_settings: &LogicSettings) {
        let speed = self.speed;

        let mut y_speed = 0.0;
        if input.up() {
            y_speed = speed;
        } else if input.down() {
            y_speed = -speed;
        }

        let mut x_speed = 0.0;
        if input.left() {
            x_speed = -speed;
        } else if input.right(){
            x_speed = speed;
        }

        self.move_position(x_speed, y_speed);

        if input.shoot() && self.laser_timer.check(PreciseTime::now(), 400) {
            let laser = Laser::new(self.data().position.x + 0.6, self.data().position.y);
            self.lasers.push(laser);
        }

        let width = logic_settings.screen_width_half - 0.5;
        let height = 4.0;
        let area = Rectangle::new(-width, width, -height, height - 1.0);
        self.stay_at_area(&area);

        self.clean_and_update_lasers(enemy, logic_settings);

        if self.circle_collision(enemy) {
            self.update_health(-logic_settings.enemy_hit_damage);
        }
    }

    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self, enemy: &mut Enemy, logic_settings: &LogicSettings) {
        let mut remove = (false, 0);

        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.update(logic_settings);

            if laser.destroy() {
                remove = (true, i);
            } else if enemy.circle_collision(laser) {
                remove = (true, i);
                enemy.update_health(-logic_settings.player_laser_damage);
            }
        }

        if let (true, i) = remove {
            self.lasers.swap_remove(i);
        }
    }

    pub fn update_health(&mut self, amount: i32) {
        self.health += amount;

        if self.health < 0 {
            self.health = 0;
        }

        println!("player health: {}", self.health);
        self.health_update = true;
    }

    pub fn health(&mut self) -> Option<u32> {
        if self.health_update {
            self.health_update = false;
            Some(self.health as u32)
        } else {
            None
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}


impl GameObject for Player {}
impl_model_matrix!(Player);


impl GameObjectData<f32> for Player {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}

pub struct Laser {
    data: Data<f32>,
    speed: f32,
    destroy: bool,
}

impl Laser {
    fn new(x: f32, y: f32) -> Laser {
        let data = Data::new(x, y, 0.3, 0.1);
        let speed = 0.08;
        let destroy = false;
        Laser { data, speed, destroy }
    }

    fn update(&mut self, logic_settings: &LogicSettings) {
        let speed = self.speed;
        self.forward(speed);

        let width = logic_settings.screen_width_half + 1.0;
        let height = 5.5;
        let area = Rectangle::new(-width, width, -height, height);

        if self.outside_allowed_area(&area) {
            self.destroy = true;
        }
    }
}

impl CanDestroy for Laser {
    fn destroy(&self) -> bool {
        self.destroy
    }
}

impl GameObjectData<f32> for Laser {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}

impl GameObject for Laser {}
impl_model_matrix!(Laser);


pub struct Enemy {
    data: Data<f32>,
    speed: f32,
    lasers: Vec<Laser>,
    laser_timer: Timer,
    health: i32,
    health_update: bool,
    visible: bool,
}

impl Enemy {
    fn new() -> Enemy {
        let data = Data::new(0.0, 0.0, 0.0, 0.0);

        let speed = 0.05;
        let lasers = vec![];
        let laser_timer = Timer::new();
        let health = 100;
        let health_update = true;
        Enemy { data, speed, lasers, laser_timer, health, health_update, visible: true }
    }

    fn reset(&mut self, logic_settings: &LogicSettings, level: u32) {
        self.data = Data::new(logic_settings.screen_width_half - 2.5, 0.0, 1.0, 1.0);
        self.lasers.clear();
        self.health = 100;
        self.health_update = true;
        self.laser_timer.reset(PreciseTime::now());
        self.visible = true;
    }

    fn update(&mut self, player: &mut Player, logic_settings: &LogicSettings) {
        let speed = self.speed;

        self.move_position(0.0, speed);

        let width = logic_settings.screen_width_half - 0.5;
        let height = 4.0;
        let area = Rectangle::new(-width, width, -height, height - 1.0);

        if self.stay_at_area(&area) {
            self.speed *= -1.0;
        }

        if self.laser_timer.check(PreciseTime::now(), 1000) {
            self.create_laser(consts::PI);
            if self.health < 25 {
                self.create_laser(consts::PI * 0.9);
                self.create_laser(consts::PI * 1.1);
            } else if self.health < 50 {
                self.create_laser(consts::PI * 0.9);
            }
        }

        self.clean_and_update_lasers(player, logic_settings);
    }

    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self, player: &mut Player, logic_settings: &LogicSettings) {
        let mut remove = (false, 0);

        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.update(logic_settings);

            if laser.destroy() {
                remove = (true, i);
            } else if player.circle_collision(laser) {
                remove = (true, i);
                player.update_health(-logic_settings.enemy_laser_damage);
            }
        }

        if let (true, i) = remove {
            self.lasers.swap_remove(i);
        }
    }

    fn create_laser(&mut self, rotation: f32) {
        let mut laser = Laser::new(self.data().position.x - 0.6, self.data().position.y);
        laser.turn(rotation);
        self.lasers.push(laser);
    }

    pub fn update_health(&mut self, amount: i32) {
        self.health += amount;

        if self.health < 0 {
            self.health = 0;
        }

        println!("enemy health: {}", self.health);
        self.health_update = true;
    }

    pub fn health(&mut self) -> Option<u32> {
        if self.health_update {
            self.health_update = false;
            Some(self.health as u32)
        } else {
            None
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}


impl GameObject for Enemy {}
impl_model_matrix!(Enemy);


impl GameObjectData<f32> for Enemy {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}


pub struct Background {
    data: Data<f32>,
    x_limit: f32,
    x_reset_position: f32,
    speed: f32,
}

impl Background {
    fn new(i: f32, side_length: f32) -> Background {
        let data = Data::new(i*side_length, 0.0, side_length, side_length);
        let x_limit = -2.0*side_length;
        let x_reset_position = 2.0*side_length;
        let speed = -0.04;

        Background { data, x_limit, x_reset_position, speed }
    }

    fn update(&mut self) {
        let speed = self.speed;
        self.move_position(speed, 0.0);

        if self.data().position.x <= self.x_limit {
            self.data_mut().position.x = self.x_reset_position;
        }
    }
}


impl GameObject for Background {}
impl_model_matrix!(Background);


impl GameObjectData<f32> for Background {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}

pub struct MovingBackground {
    backgrounds: [Background; 4],
}

impl MovingBackground {
    pub fn new() -> MovingBackground {
        let size = 9.0;
        let backgrounds = [
            Background::new(-1.0, size),
            Background::new(0.0, size),
            Background::new(1.0, size),
            Background::new(2.0, size),
        ];

        MovingBackground { backgrounds }
    }

    fn update(&mut self) {
       for background in &mut self.backgrounds {
           background.update();
       }
    }

    pub fn get_backgrounds(&self) -> &[Background; 4] {
        &self.backgrounds
    }

    pub fn move_position_x(&mut self, x: f32) {
        for background in self.backgrounds.iter_mut() {
            background.move_position(x, 0.0);
        }
    }
}