/*
src/logic/mod.rs, 2017-08-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

pub mod common;

use std::f32::consts;
use std::convert::From;

use cgmath::{Matrix4, Vector2, vec2};
use cgmath::prelude::*;

use rand::{Rng, ThreadRng};
use rand;

use logic::common::*;

use input::Input;

use time::PreciseTime;
use Timer;

use gui::{GUI, GUIState, GUIEvent};

use renderer::{ModelMatrix, SCREEN_TOP_Y_VALUE_IN_WORLD_COORDINATES};

use audio::{SoundEffectManager, SoundEffectPlayer};

const BACKGROUND_MOVING_SPEED: f32 = -0.02;
const BACKGROUND_SQUARE_SIDE_LENGTH: f32 = 9.0;

const PLAYER_MOVEMENT_SPEED: f32 = 0.05;
const PLAYER_SQUARE_SIDE_LENGTH: f32 = 1.0;
const PLAYER_SQUARE_SIDE_LENGTH_HALF: f32 = PLAYER_SQUARE_SIDE_LENGTH/2.0;
const PLAYER_STARTING_POSITION: Vector2<f32> = Vector2 { x: -3.0, y: 0.0 };
pub const PLAYER_MAX_HEALTH: i32 = 100;
const PLAYER_MILLISECONDS_BETWEEN_LASERS: i64 = 400;

const LAST_LEVEL_INDEX: u32 = 3;

const PARTICLE_SQUARE_SIDE_LENGTH: f32 = 0.1;
const EXPLOSION_PARTICLE_COUNT: u32 = 15;
const EXPLOSION_MILLISECONDS_BETWEEN_PARTICLE_CREATION: i64 = 500;
const EXPLOSION_VISIBILITY_TIME_MILLISECONDS: i64 = 1500;

const FULL_CIRCLE_ANGLE_IN_RADIANS: f32 = consts::PI*2.0;

const LASER_SPEED: f32 = 0.08;

const ENEMY_MOVEMENT_SPEED: f32 = 0.04;
pub const ENEMY_MAX_HEALTH: i32 = 100;
const ENEMY_SQUARE_SIDE_LENGTH: f32 = 1.0;
const ENEMY_SQUARE_SIDE_LENGTH_HALF: f32 = ENEMY_SQUARE_SIDE_LENGTH/2.0;
const ENEMY_MILLISECONDS_BETWEEN_LASERS: i64 = 1000;
const ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS: i64 = 3000;

const GUI_MARGIN_TOP: f32 = 1.0;

macro_rules! impl_traits {
    ( $x:ty ) => {
        impl GameObject for $x {}

        impl ModelMatrix for $x {
            fn model_matrix(&self) -> &Matrix4<f32> {
                &self.data().model_matrix
            }
        }

        impl GameObjectData<f32> for $x {
            fn data(&self) -> &Data<f32> {
                &self.data
            }
            fn data_mut(&mut self) -> &mut Data<f32> {
                &mut self.data
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

#[derive(Copy, Clone)]
pub enum LaserColor {
    Red,
    Green,
    Blue,
}

#[derive(Copy, Clone)]
pub enum EnemyType {
    Normal,
    Shield,
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
            explosion: Explosion::new(EXPLOSION_PARTICLE_COUNT, EXPLOSION_MILLISECONDS_BETWEEN_PARTICLE_CREATION),
        }
    }

    pub fn update<T: Input>(&mut self, input: &T, gui: &mut GUI, sound_effect_manager: &mut SoundEffectManager) {

        if self.game_running {
            self.player.update(input, &mut self.enemy, &self.logic_settings, sound_effect_manager);
            self.enemy.update(&mut self.player, &self.logic_settings, sound_effect_manager);
            self.moving_background.update();
        }

        self.explosion.update(sound_effect_manager);

        if let Some(health) = self.player.health() {
            gui.get_game_status().set_player_health(health);

            if health == 0 {
                self.player.lasers.clear();
                self.enemy.lasers.clear();
                self.enemy.laser_bombs.clear();

                self.game_running = false;
                self.explosion.start_explosion(&self.player);
            }
        }

        if let Some(health) = self.enemy.health() {
            gui.get_game_status().set_enemy_health(health);

            if health == 0 {
                self.player.lasers.clear();
                self.enemy.lasers.clear();
                self.enemy.laser_bombs.clear();

                self.game_running = false;
                self.explosion.start_explosion(&self.enemy);
            }
        }

        if !self.game_running && self.explosion.explosion_finished() {
            if self.player.health == 0 {
                gui.handle_gui_event(GUIEvent::ChangeState(GUIState::GameOverScreen));
                self.player.visible = false;
            } else {
                if self.level == LAST_LEVEL_INDEX {
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
        if level > LAST_LEVEL_INDEX {
            panic!("level index must be at range 0-{}", LAST_LEVEL_INDEX);
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

        self.explosion.visible = false;
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
    fn new(current_time: PreciseTime, position: Vector2<f32>, angle: f32, speed: f32, lifetime_as_milliseconds: i64) -> Particle {

        let mut particle = Particle {
            data: Data::new_square(position, PARTICLE_SQUARE_SIDE_LENGTH),
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

impl_traits!(Particle);


pub struct Explosion {
    position: Vector2<f32>,
    visible: bool,
    timer: Timer,
    particles: Vec<Particle>,
    particle_creation_timer: Timer,
    rng: ThreadRng,
    remove_at_index: Vec<usize>,
    particle_count: u32,
    milliseconds_between_particle_generation: i64,
}

impl Explosion {
    fn new(particle_count: u32, milliseconds_between_particle_generation: i64) -> Explosion {
        Explosion {
            position: Vector2::zero(),
            visible: false,
            timer: Timer::new(),
            particles: Vec::new(),
            particle_creation_timer: Timer::new(),
            rng: rand::thread_rng(),
            remove_at_index: Vec::new(),
            particle_count,
            milliseconds_between_particle_generation,
        }
    }

    pub fn start_explosion<T: GameObject>(&mut self, object: &T) {
        let current_time = PreciseTime::now();
        self.timer.reset(current_time);
        self.position = *object.position();
        self.visible = true;
        self.particles.clear();
    }

    pub fn explosion_finished(&mut self) -> bool {
        if self.timer.check(PreciseTime::now(), EXPLOSION_VISIBILITY_TIME_MILLISECONDS) {
            self.visible = false;
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, sounds: &mut SoundEffectManager) {
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

        if self.particle_creation_timer.check(current_time, self.milliseconds_between_particle_generation) {
            sounds.explosion();
            for _ in 0..self.particle_count {
                self.particles.push(Particle::new(current_time, self.position, FULL_CIRCLE_ANGLE_IN_RADIANS * self.rng.gen::<f32>(), (self.rng.gen::<f32>()*0.02).max(0.01), (self.rng.gen::<u32>()%400+500) as i64));
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
        Player {
            data: Data::new_square(Vector2::zero(), PLAYER_SQUARE_SIDE_LENGTH),
            speed: PLAYER_MOVEMENT_SPEED,
            lasers: vec![],
            laser_timer: Timer::new(),
            health: PLAYER_MAX_HEALTH,
            health_update: true,
            visible: true,
        }
    }

    fn reset(&mut self, logic_settings: &LogicSettings) {
        self.data = Data::new_square(PLAYER_STARTING_POSITION, PLAYER_SQUARE_SIDE_LENGTH);
        self.lasers.clear();
        self.health = PLAYER_MAX_HEALTH;
        self.health_update = true;
        self.laser_timer.reset(PreciseTime::now());
        self.visible = true;
    }

    fn update(&mut self, input: &Input, enemy: &mut Enemy, logic_settings: &LogicSettings, sounds: &mut SoundEffectManager) {
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

        if input.shoot() && self.laser_timer.check(PreciseTime::now(), PLAYER_MILLISECONDS_BETWEEN_LASERS) {
            sounds.laser();
            let position = Vector2::new(self.x() + 0.6, self.y());
            let laser = Laser::new(position, LaserColor::Green);
            self.lasers.push(laser);
        }

        let width = logic_settings.screen_width_half - PLAYER_SQUARE_SIDE_LENGTH_HALF;
        let height = SCREEN_TOP_Y_VALUE_IN_WORLD_COORDINATES - PLAYER_SQUARE_SIDE_LENGTH_HALF;
        let area = Rectangle::new(-width, width, -height, height - GUI_MARGIN_TOP);
        self.stay_at_area(&area);

        self.clean_and_update_lasers(enemy, logic_settings, sounds);

        if self.circle_collision(enemy) {
            self.update_health(-logic_settings.enemy_hit_damage);
        }
    }

    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self, enemy: &mut Enemy, logic_settings: &LogicSettings, sounds: &mut SoundEffectManager) {
        let mut remove = (false, 0);

        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.update(logic_settings);

            if laser.destroy() {
                remove = (true, i);
            } else {
                if let EnemyType::Shield = enemy.enemy_type {
                    if enemy.shield.visible && enemy.shield.circle_collision(laser) {
                        remove = (true, i);
                    } else if enemy.laser_cannon_bottom.circle_collision(laser) {
                        if enemy.laser_cannon_bottom.parent_object_shield_enabled {
                            sounds.player_laser_hits_laser_cannon();
                        }
                        enemy.laser_cannon_bottom.parent_object_shield_enabled = false;
                        remove = (true, i);
                    } else if enemy.laser_cannon_top.circle_collision(laser) {
                        if enemy.laser_cannon_top.parent_object_shield_enabled {
                            sounds.player_laser_hits_laser_cannon();
                        }
                        enemy.laser_cannon_top.parent_object_shield_enabled = false;
                        remove = (true, i);
                    } else if !enemy.shield.visible && enemy.circle_collision(laser)  {
                        remove = (true, i);
                        enemy.update_health(-logic_settings.player_laser_damage);
                    }
                } else {
                    if enemy.circle_collision(laser) {
                        remove = (true, i);
                        enemy.update_health(-logic_settings.player_laser_damage);
                    }
                }
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

impl_traits!(Player);


pub struct Laser {
    data: Data<f32>,
    speed: f32,
    destroy: bool,
    color: LaserColor,
}

impl Laser {
    fn new(position: Vector2<f32>, color: LaserColor) -> Laser {
        Laser {
            data: Data::new(position, 0.3, 0.1),
            speed: LASER_SPEED,
            destroy: false,
            color: color,
        }
    }

    fn new_with_width_and_height(position: Vector2<f32>, color: LaserColor, width: f32, height: f32) -> Laser {
        let data = Data::new(position, width, height);
        let speed = 0.08;
        let destroy = false;
        Laser { data, speed, destroy, color }
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

    pub fn color(&self) -> LaserColor {
        self.color
    }
}

impl CanDestroy for Laser {
    fn destroy(&self) -> bool {
        self.destroy
    }
}

impl_traits!(Laser);


pub struct Enemy {
    data: Data<f32>,
    speed: f32,
    lasers: Vec<Laser>,
    laser_timer: Timer,
    health: i32,
    health_update: bool,
    visible: bool,
    enemy_type: EnemyType,
    laser_cannon_top: LaserCannon,
    laser_cannon_bottom: LaserCannon,
    laser_bombs: Vec<LaserBomb>,
    laser_bomb_timer: Timer,
    laser_bomb_enabled: bool,
    shield: Shield,
    laser_x_position_margin: f32,
}

impl Enemy {
    fn new() -> Enemy {
        Enemy {
            data: Data::new_square(Vector2::zero(), 0.0),
            speed: ENEMY_MOVEMENT_SPEED,
            lasers: vec![],
            laser_timer: Timer::new(),
            health: ENEMY_MAX_HEALTH,
            health_update: true,
            visible: true,
            enemy_type: EnemyType::Normal,
            laser_cannon_top: LaserCannon::new(true),
            laser_cannon_bottom: LaserCannon::new(false),
            laser_bombs: Vec::new(),
            laser_bomb_timer: Timer::new(),
            laser_bomb_enabled: true,
            shield: Shield::new(Vector2::zero()),
            laser_x_position_margin: -0.6,
        }
    }

    fn reset(&mut self, logic_settings: &LogicSettings, level: u32) {
        self.data = Data::new_square(vec2(logic_settings.screen_width_half - 2.5, 0.0), ENEMY_SQUARE_SIDE_LENGTH);
        self.lasers.clear();
        self.health = ENEMY_MAX_HEALTH;
        self.health_update = true;

        let time = PreciseTime::now();

        self.laser_bomb_timer.reset(time);
        self.laser_timer.reset(time);
        self.visible = true;

        if level == 0 || level == 2 {
            self.enemy_type = EnemyType::Normal;
        } else {
            self.enemy_type = EnemyType::Shield;
        }

        if level >= 2 {
            self.laser_bomb_enabled = true;
        } else {
            self.laser_bomb_enabled = false;
        }
        self.laser_bombs.clear();

        self.laser_cannon_bottom.reset(self.data.position, level, time);
        self.laser_cannon_top.reset(self.data.position, level, time);
        self.shield.reset(self.data.position, level);
    }

    fn update(&mut self, player: &mut Player, logic_settings: &LogicSettings, sounds: &mut SoundEffectManager) {
        let speed = self.speed;

        self.move_position(0.0, speed);

        let width = logic_settings.screen_width_half - ENEMY_SQUARE_SIDE_LENGTH_HALF;
        let height = if let EnemyType::Shield = self.enemy_type {
            2.0
        } else {
            4.0
        };

        let area = Rectangle::new(-width, width, -height, height - GUI_MARGIN_TOP);

        if self.stay_at_area(&area) {
            self.speed *= -1.0;
        }

        let current_time = PreciseTime::now();

        if self.laser_timer.check(current_time, ENEMY_MILLISECONDS_BETWEEN_LASERS) {
            self.create_laser(consts::PI);
            if self.health < 25 {
                self.create_laser(consts::PI * 0.9);
                self.create_laser(consts::PI * 1.1);
            } else if self.health < 50 {
                self.create_laser(consts::PI * 0.9);
            }
        }

        self.clean_and_update_lasers(player, logic_settings);

        let y = self.y();
        self.laser_cannon_bottom.update(y, current_time, logic_settings, &mut self.lasers);
        self.laser_cannon_top.update(y, current_time, logic_settings, &mut self.lasers);

        if self.laser_bomb_enabled {
            self.clean_and_update_laser_bombs(player, logic_settings, current_time, sounds);

            if self.laser_bomb_timer.check(current_time, ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS) {
                sounds.laser_bomb_launch();
                self.create_laser_bomb();
            }
        }

        if let EnemyType::Shield = self.enemy_type {
            if self.shield.update(self.data.position.y, current_time) {
                self.laser_cannon_top.parent_object_shield_enabled = true;
                self.laser_cannon_bottom.parent_object_shield_enabled = true;
            }

            if self.shield.visible && !self.laser_cannon_top.parent_object_shield_enabled && !self.laser_cannon_bottom.parent_object_shield_enabled {
                self.shield.disable(current_time);
            }
        }

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

    fn clean_and_update_laser_bombs(&mut self, player: &mut Player, logic_settings: &LogicSettings, time: PreciseTime, sounds: &mut SoundEffectManager) {
        let mut remove = (false, 0);

        for (i, laser_bomb) in self.laser_bombs.iter_mut().enumerate() {
            laser_bomb.update(time, logic_settings, &mut self.lasers, sounds);

            if laser_bomb.destroy() {
                remove = (true, i);
            } else if player.circle_collision(laser_bomb) {
                remove = (true, i);
                player.update_health(-logic_settings.enemy_laser_damage);
            }
        }

        if let (true, i) = remove {
            self.laser_bombs.swap_remove(i);
        }
    }

    fn create_laser(&mut self, rotation: f32) {
        let position = vec2(self.x() + self.laser_x_position_margin, self.y());
        let mut laser = Laser::new(position, LaserColor::Red);
        laser.turn(rotation);
        self.lasers.push(laser);
    }

    fn create_laser_bomb(&mut self) {
        let mut laser_bomb = match self.enemy_type {
            EnemyType::Normal => {
                LaserBomb::new(vec2(self.x() + self.laser_x_position_margin, self.y()))
            },
            EnemyType::Shield => {

                if self.laser_cannon_top.laser_bombs_enabled {
                    self.laser_cannon_top.laser_bombs_enabled = false;
                    let position = vec2(self.laser_cannon_top.x() + self.laser_x_position_margin, self.laser_cannon_top.y());
                    LaserBomb::new(position)
                } else {
                    self.laser_cannon_top.laser_bombs_enabled = true;
                    let position = vec2(self.laser_cannon_bottom.x() + self.laser_x_position_margin, self.laser_cannon_bottom.y());
                    LaserBomb::new(position)
                }
            },
        };

        laser_bomb.turn(consts::PI);
        self.laser_bombs.push(laser_bomb);
    }

    pub fn update_health(&mut self, amount: i32) {
        self.health += amount;

        if self.health < 0 {
            self.health = 0;
        }

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

    pub fn get_laser_cannon_top(&self) -> &LaserCannon {
        &self.laser_cannon_top
    }

    pub fn get_laser_cannon_bottom(&self) -> &LaserCannon {
        &self.laser_cannon_bottom
    }

    pub fn get_laser_bombs(&self) -> &Vec<LaserBomb> {
        &self.laser_bombs
    }

    pub fn get_shield(&self) -> &Shield {
        &self.shield
    }
}

impl_traits!(Enemy);



pub struct Shield {
    data: Data<f32>,
    visible: bool,
    timer: Timer,
}

impl Shield {
    fn new(position: Vector2<f32>) -> Shield {
        let size = 1.5;
        Shield {
            data: Data::new_square(position, size),
            visible: false,
            timer: Timer::new(),
        }
    }

    fn reset(&mut self, parent_position: Vector2<f32>, level: u32) {
        if level == 1 || level == 3 {
            self.visible = true;
        } else {
            self.visible = false;
            return;
        }

        self.set_position(parent_position.x, 0.0);
        self.update_position(parent_position.y);
    }

    fn update(&mut self, parent_position_y: f32, current_time: PreciseTime) -> bool {
        self.update_position(parent_position_y);

        if !self.visible && self.timer.check(current_time, 10_000) {
            self.visible = true;
            true
        } else {
            false
        }
    }

    fn update_position(&mut self, parent_object_y: f32) {
        self.set_position_y(parent_object_y);
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn disable(&mut self, current_time: PreciseTime) {
        self.timer.reset(current_time);
        self.visible = false;
    }
}

impl_traits!(Shield);


pub struct LaserCannon {
    data: Data<f32>,
    cannon_position_top: bool,
    visible: bool,
    laser_bombs_enabled: bool,
    laser_timer: Timer,
    parent_object_shield_enabled: bool,
}

impl LaserCannon {
    fn new(cannon_position_top: bool) -> LaserCannon {
        let size = 0.7;

        LaserCannon {
            data: Data::new_square(Vector2::zero(), size),
            cannon_position_top,
            visible: false,
            laser_bombs_enabled: false,
            laser_timer: Timer::new(),
            parent_object_shield_enabled: true,
        }
    }

    fn reset(&mut self, parent_position: Vector2<f32>, level: u32, current_time: PreciseTime) {
        if level == 1 || level == 3 {
            self.visible = true;
        } else {
            self.visible = false;
            return;
        }

        self.set_position(parent_position.x, 0.0);
        self.update_position(parent_position.y);

        if level == 3 && self.cannon_position_top {
            self.laser_bombs_enabled = true;
        } else {
            self.laser_bombs_enabled = false;
        }

        self.laser_timer.reset(current_time);

        self.parent_object_shield_enabled = true;
    }

    fn update(&mut self, parent_position_y: f32, current_time: PreciseTime, logic_settings: &LogicSettings, parents_lasers: &mut Vec<Laser>) {
        if !self.visible {
            return;
        }

        if self.laser_timer.check(current_time, 1000) {
            let position = vec2(self.x() - 0.5, self.y());
            let mut laser = Laser::new(position, LaserColor::Red);
            laser.turn(consts::PI);
            parents_lasers.push(laser);
        }

        self.update_position(parent_position_y);
    }

    fn update_position(&mut self, parent_object_y: f32) {
        if self.cannon_position_top {
            self.set_position_y(parent_object_y + 2.0);
        } else {
            self.set_position_y(parent_object_y - 2.0);
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn green_color(&self) -> bool {
        self.parent_object_shield_enabled
    }
}

impl_traits!(LaserCannon);


pub struct LaserBomb {
    laser: Laser,
    timer: Timer,
}

impl LaserBomb {
    fn new(position: Vector2<f32>) -> LaserBomb {
        let size = 0.25;
        LaserBomb {
            laser: Laser::new_with_width_and_height(position, LaserColor::Blue, size, size),
            timer: Timer::new(),
        }
    }

    fn update(&mut self, current_time: PreciseTime, logic_settings: &LogicSettings, parent_lasers: &mut Vec<Laser>, sounds: &mut SoundEffectManager) {
        self.laser.update(logic_settings);

        if self.timer.check(current_time, 1000) {
            sounds.laser_bomb_explosion();
            let laser_count : u16 = 15;
            let mut angle = 0.0;
            let angle_between_lasers = (consts::PI*2.0) / f32::from(laser_count);

            for _ in 0..laser_count {
                let position = vec2(self.x() - 0.6, self.y());
                let mut laser = Laser::new(position, LaserColor::Blue);
                laser.turn(angle);
                parent_lasers.push(laser);

                angle += angle_between_lasers;
            }

            self.laser.destroy = true;
        }
    }
}

impl CanDestroy for LaserBomb {
    fn destroy(&self) -> bool {
        self.laser.destroy()
    }
}

impl GameObject for LaserBomb {}

impl ModelMatrix for LaserBomb {
    fn model_matrix(&self) -> &Matrix4<f32> {
        &self.data().model_matrix
    }
}

impl GameObjectData<f32> for LaserBomb {
    fn data(&self) -> &Data<f32> {
        &self.laser.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.laser.data
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
        let data = Data::new_square(vec2(i*side_length, 0.0), side_length);
        let x_limit = -2.0*side_length;
        let x_reset_position = 2.0*side_length;
        let speed = BACKGROUND_MOVING_SPEED;

        Background { data, x_limit, x_reset_position, speed }
    }

    fn update(&mut self) {
        let speed = self.speed;
        self.move_position(speed, 0.0);

        if self.x() <= self.x_limit {
            self.data_mut().position.x = self.x_reset_position;
        }
    }
}


impl_traits!(Background);

pub struct MovingBackground {
    backgrounds: [Background; 4],
}

impl MovingBackground {
    pub fn new() -> MovingBackground {
        let size = BACKGROUND_SQUARE_SIDE_LENGTH;
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