/*
src/logic/mod.rs, 2017-08-22

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Game logic.

pub mod common;

use std::f32::consts;
use std::convert::From;

use cgmath::{Matrix4, Vector2, vec2};
use cgmath::prelude::*;

use rand::{Rng, ThreadRng};
use rand;

use logic::common::*;

use input::Input;

use utils::{Timer, GameTimeManager};

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
const PLAYER_MILLISECONDS_BETWEEN_LASERS: u32 = 300;

const LAST_LEVEL_INDEX: u32 = 3;

const PARTICLE_SQUARE_SIDE_LENGTH: f32 = 0.1;
const EXPLOSION_PARTICLE_COUNT: u32 = 15;
const EXPLOSION_MILLISECONDS_BETWEEN_PARTICLE_CREATION: u32 = 500;
const EXPLOSION_VISIBILITY_TIME_MILLISECONDS: u32 = 2000;

const FULL_CIRCLE_ANGLE_IN_RADIANS: f32 = consts::PI*2.0;

const LASER_SPEED: f32 = 0.08;

const ENEMY_MOVEMENT_SPEED: f32 = 0.04;
const ENEMY_WITH_SHIELD_MOVEMENT_SPEED: f32 = 0.02;
pub const ENEMY_MAX_HEALTH: i32 = 100;
const ENEMY_SQUARE_SIDE_LENGTH: f32 = 1.0;
const ENEMY_SQUARE_SIDE_LENGTH_HALF: f32 = ENEMY_SQUARE_SIDE_LENGTH/2.0;
const ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS_BEGINNING: u32 = 3750;
const ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS_HEALTH_40: u32 = 2500;
const ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS_HEALTH_20: u32 = 1250;

const LASER_BOMB_DAMAGE: i32 = 30;
const LASER_BOMB_EXPLOSION_TIME_MILLISECONDS: u32 = 900;

const LASER_CANNON_DISTANCE_FROM_ENEMY: f32 = 3.0;

const GUI_MARGIN_TOP: f32 = 1.0;

/// Macro for implementing basic game object traits.
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

/// Game's difficulty levels.
#[derive(Copy, Clone, PartialEq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

/// Laser colors.
#[derive(Copy, Clone)]
pub enum LaserColor {
    Red,
    Green,
    Blue,
}

/// Current mode of the Enemy game object.
#[derive(Copy, Clone)]
pub enum EnemyType {
    Normal,
    Shield,
}

/// Settings depending on current game difficulty.
struct LogicSettings {
    screen_width_half: f32,
    player_laser_damage: i32,
    enemy_laser_damage: i32,
    enemy_hit_damage_16_milliseconds: i32,
    enemy_shooting_speed_milliseconds: u32,
    difficulty: Difficulty,
}

impl LogicSettings {
    /// Create new `LogicSettings`.
    fn new() -> LogicSettings {
        LogicSettings {
            screen_width_half: 0.0,
            player_laser_damage: 0,
            enemy_laser_damage: 0,
            enemy_hit_damage_16_milliseconds: 0,
            enemy_shooting_speed_milliseconds: 0,
            difficulty: Difficulty::Normal,
        }
    }

    /// Set settings for difficulty level easy.
    fn settings_easy(&mut self) {
        self.player_laser_damage = 5;
        self.enemy_laser_damage = 5;
        self.enemy_hit_damage_16_milliseconds = 3;
        self.enemy_shooting_speed_milliseconds = 1500;
        self.difficulty = Difficulty::Easy;
    }

    /// Set settings for difficulty level normal.
    fn settings_normal(&mut self) {
        self.player_laser_damage = 3;
        self.enemy_laser_damage = 10;
        self.enemy_hit_damage_16_milliseconds = 6;
        self.enemy_shooting_speed_milliseconds = 1000;
        self.difficulty = Difficulty::Normal;
    }

    /// Set settings for difficulty level hard.
    fn settings_hard(&mut self) {
        self.player_laser_damage = 2;
        self.enemy_laser_damage = 10;
        self.enemy_hit_damage_16_milliseconds = 6;
        self.enemy_shooting_speed_milliseconds = 750;
        self.difficulty = Difficulty::Hard;
    }
}

/// Logic stores current state of game logic.
pub struct Logic {
    player: Player,
    enemy: Enemy,
    moving_background: MovingBackground,
    logic_settings: LogicSettings,
    level: u32,
    current_difficulty: Difficulty,
    game_running: bool,
    explosion: Explosion,
    index_buffer: Vec<usize>,
}

impl Logic {
    /// Create new `Logic`.
    pub fn new() -> Logic {
        let mut logic = Logic {
            player: Player::new(),
            enemy: Enemy::new(),
            moving_background: MovingBackground::new(),
            logic_settings: LogicSettings::new(),
            level: 0,
            current_difficulty: Difficulty::Normal,
            game_running: true,
            explosion: Explosion::new(EXPLOSION_PARTICLE_COUNT, EXPLOSION_MILLISECONDS_BETWEEN_PARTICLE_CREATION),
            index_buffer: Vec::with_capacity(25),
        };

        // Move background star behind "Settings" text.
        logic.moving_background.move_position_x(0.05);

        logic
    }

    /// Updates game logic.
    pub fn update<T: Input>(&mut self, input: &T, gui: &mut GUI, sound_effect_manager: &mut SoundEffectManager, current_time: &GameTimeManager) {

        // Basic game updating.

        if self.game_running {
            self.player.update(input, &mut self.enemy, &self.logic_settings, sound_effect_manager, &mut self.index_buffer, current_time);
            self.enemy.update(&mut self.player, &self.logic_settings, sound_effect_manager, &mut self.index_buffer, current_time);
            self.moving_background.update(current_time);
        }

        // Handle game ending and health updates to GUI.

        self.explosion.update(sound_effect_manager, &mut self.index_buffer, current_time);

        if let Some(health) = self.player.health() {
            gui.get_game_status().set_player_health(health);

            if health == 0 {
                self.player.lasers.clear();
                self.enemy.lasers.clear();
                self.enemy.laser_bombs.clear();

                self.game_running = false;
                self.explosion.start_explosion(&self.player, current_time);
            }
        }

        if let Some(health) = self.enemy.health() {
            gui.get_game_status().set_enemy_health(health);

            if health == 0 {
                self.player.lasers.clear();
                self.enemy.lasers.clear();
                self.enemy.laser_bombs.clear();

                self.game_running = false;
                self.explosion.start_explosion(&self.enemy, current_time);
            }
        }

        if !self.game_running && self.explosion.explosion_finished(current_time) {
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

    /// Get player.
    pub fn get_player(&self) -> &Player {
        &self.player
    }

    /// Get enemy.
    pub fn get_enemy(&self) -> &Enemy {
        &self.enemy
    }

    /// Get explosion.
    pub fn get_explosion(&self) -> &Explosion {
        &self.explosion
    }

    /// Get background.
    pub fn get_moving_background(&self) -> &MovingBackground {
        &self.moving_background
    }

    /// Resets game logic to specific level and difficulty level.
    ///
    /// # Panics
    /// If argument level is greater than LAST_LEVEL_INDEX.
    pub fn reset_game(&mut self, gui: &mut GUI, difficulty: Difficulty, level: u32, current_time: &GameTimeManager) {
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

        self.player.reset(current_time);
        self.enemy.reset(&self.logic_settings, level, current_time);

        if let Some(health) = self.player.health() {
            gui.get_game_status().set_player_health(health);
        }

        if let Some(health) = self.enemy.health() {
            gui.get_game_status().set_enemy_health(health);
        }

        self.explosion.reset();
    }

    /// Change to next level and reset game.
    pub fn reset_to_next_level(&mut self, gui: &mut GUI, current_time: &GameTimeManager) {
        let difficulty = self.current_difficulty;
        let level = self.level + 1;
        self.reset_game(gui, difficulty, level, current_time);
    }

    /// Updates game world width.
    pub fn update_half_screen_width(&mut self, half_width: f32) {
        self.logic_settings.screen_width_half = half_width;
    }
}

/// Explosion particle.
pub struct Particle {
    data: Data<f32>,
    speed: f32,
    lifetime_timer: Timer,
    lifetime_as_milliseconds: u32,
}

impl Particle {
    /// Create new `Particle`.
    fn new(current_time: &GameTimeManager, position: Vector2<f32>, angle: f32, speed: f32, lifetime_as_milliseconds: u32) -> Particle {
        let mut particle = Particle {
            data: Data::new_square(position, PARTICLE_SQUARE_SIDE_LENGTH),
            speed,
            lifetime_timer: Timer::new_from_time(current_time.time()),
            lifetime_as_milliseconds,
        };
        particle.turn_without_updating_model_matrix(angle);

        particle
    }

    /// Updates particle and returns true if particle can be destroyed.
    fn update(&mut self, current_time: &GameTimeManager) -> bool {
        let speed = self.speed;
        self.forward(speed * current_time.delta_time());

        self.lifetime_timer.check(current_time.time(), self.lifetime_as_milliseconds)
    }
}

impl_traits!(Particle);


/// Explosion manages particles and creates them.
pub struct Explosion {
    position: Vector2<f32>,
    visible: bool,
    timer: Timer,
    particles: Vec<Particle>,
    particle_creation_timer: Timer,
    rng: ThreadRng,
    particle_count: u32,
    milliseconds_between_particle_generation: u32,
}

impl Explosion {
    /// Create new `Explosion`.
    fn new(particle_count: u32, milliseconds_between_particle_generation: u32) -> Explosion {
        Explosion {
            position: Vector2::zero(),
            visible: false,
            timer: Timer::new(),
            particles: Vec::with_capacity(25),
            particle_creation_timer: Timer::new(),
            rng: rand::thread_rng(),
            particle_count,
            milliseconds_between_particle_generation,
        }
    }

    /// Moves explosion to location of argument game object, and starts explosion.
    pub fn start_explosion<T: GameObject>(&mut self, object: &T, current_time: &GameTimeManager) {
        self.timer.reset(current_time.time());
        self.position = *object.position();
        self.visible = true;
        self.particles.clear();
    }

    /// Return true if explosion is finished.
    pub fn explosion_finished(&mut self, current_time: &GameTimeManager) -> bool {
        if self.timer.check(current_time.time(), EXPLOSION_VISIBILITY_TIME_MILLISECONDS) {
            self.visible = false;
            true
        } else {
            false
        }
    }

    /// If explosion is visible, update current particles and create new particles if its time to create particles.
    pub fn update(&mut self, sounds: &mut SoundEffectManager, index_buffer: &mut Vec<usize>, current_time: &GameTimeManager) {
        if !self.visible {
            return;
        }

        self.particles.update(index_buffer, &mut | particle | {
            particle.update(current_time)
        });

        if self.particle_creation_timer.check(current_time.time(), self.milliseconds_between_particle_generation) {
            sounds.explosion();
            for _ in 0..self.particle_count {
                self.particles.push(Particle::new(current_time, self.position, FULL_CIRCLE_ANGLE_IN_RADIANS * self.rng.gen::<f32>(), (self.rng.gen::<f32>()*0.02).max(0.01), self.rng.gen::<u32>()%400+500));
            }
        }
    }

    /// Hides explosion.
    pub fn reset(&mut self) {
        self.visible = false;
    }

    /// Return true if explosion is visible.
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Get current particles.
    pub fn particles(&self) -> &Vec<Particle> {
        &self.particles
    }
}


/// Player game object and logic.
pub struct Player {
    data: Data<f32>,
    speed: f32,
    lasers: Vec<Laser>,
    laser_timer: Timer,
    health: i32,
    health_update: bool,
    visible: bool,
    enemy_hit_damage_timer: Timer,
}

impl Player {
    /// Create new `Player`.
    fn new() -> Player {
        Player {
            data: Data::new_square(Vector2::zero(), PLAYER_SQUARE_SIDE_LENGTH),
            speed: PLAYER_MOVEMENT_SPEED,
            lasers: Vec::with_capacity(25),
            laser_timer: Timer::new(),
            health: PLAYER_MAX_HEALTH,
            health_update: true,
            visible: true,
            enemy_hit_damage_timer: Timer::new(),
        }
    }

    /// Reset player's state and position player to start position.
    fn reset(&mut self, current_time: &GameTimeManager) {
        self.data = Data::new_square(PLAYER_STARTING_POSITION, PLAYER_SQUARE_SIDE_LENGTH);
        self.lasers.clear();
        self.health = PLAYER_MAX_HEALTH;
        self.health_update = true;
        self.laser_timer.reset(current_time.time());
        self.visible = true;
    }

    /// Updates player logic.
    fn update(&mut self,
            input: &Input,
            enemy: &mut Enemy,
            logic_settings: &LogicSettings,
            sounds: &mut SoundEffectManager,
            index_buffer: &mut Vec<usize>,
            current_time: &GameTimeManager) {
        // Move player.

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

        self.move_position(x_speed*current_time.delta_time(), y_speed*current_time.delta_time());

        // Keep player on the screen.

        let width = logic_settings.screen_width_half - PLAYER_SQUARE_SIDE_LENGTH_HALF;
        let height = SCREEN_TOP_Y_VALUE_IN_WORLD_COORDINATES - PLAYER_SQUARE_SIDE_LENGTH_HALF;
        let area = Rectangle::new(-width, width, -height, height - GUI_MARGIN_TOP);
        self.stay_at_area(&area);

        // Create new laser if player shoots.

        if input.shoot() && self.laser_timer.check(current_time.time(), PLAYER_MILLISECONDS_BETWEEN_LASERS) {
            sounds.laser();
            let position = Vector2::new(self.x() + 0.5, self.y());
            let laser = Laser::new(position, LaserColor::Green);
            self.lasers.push(laser);
        }

        // Update player lasers.

        self.clean_and_update_lasers(enemy, logic_settings, sounds, index_buffer, current_time);

        // Check if there is collision between player and enemy.

        if self.enemy_hit_damage_timer.check(current_time.time(), 16) {
            if self.circle_collision(enemy) {
                self.update_health(-logic_settings.enemy_hit_damage_16_milliseconds);
            }

            if let EnemyType::Shield = enemy.enemy_type {
                if self.circle_collision(enemy.get_laser_cannon_top()) {
                    self.update_health(-logic_settings.enemy_hit_damage_16_milliseconds);
                } else if self.circle_collision(enemy.get_laser_cannon_bottom()) {
                    self.update_health(-logic_settings.enemy_hit_damage_16_milliseconds);
                }
            }
        }
    }

    /// Get player's lasers.
    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self,
            enemy: &mut Enemy,
            logic_settings: &LogicSettings,
            sounds: &mut SoundEffectManager,
            index_buffer: &mut Vec<usize>,
            current_time: &GameTimeManager) {
        self.lasers.update(index_buffer, &mut |laser| {
            laser.update(logic_settings, current_time);

            if laser.destroy() {
                return true;
            }

            // Check if there is collision between enemy and the laser.

            if let EnemyType::Shield = enemy.enemy_type {
                if enemy.shield.visible && enemy.shield.circle_collision(laser) {
                    true
                } else if enemy.laser_cannon_bottom.circle_collision(laser) {
                    if enemy.laser_cannon_bottom.parent_object_shield_enabled {
                        sounds.player_laser_hits_laser_cannon();
                    }
                    enemy.laser_cannon_bottom.parent_object_shield_enabled = false;
                    true
                } else if enemy.laser_cannon_top.circle_collision(laser) {
                    if enemy.laser_cannon_top.parent_object_shield_enabled {
                        sounds.player_laser_hits_laser_cannon();
                    }
                    enemy.laser_cannon_top.parent_object_shield_enabled = false;
                    true
                } else if !enemy.shield.visible && enemy.circle_collision(laser)  {
                    enemy.update_health(-logic_settings.player_laser_damage);
                    true
                } else {
                    false
                }
            } else {
                if enemy.circle_collision(laser) {
                    enemy.update_health(-logic_settings.player_laser_damage);
                    return true
                } else {
                    false
                }
            }
        });
    }

    /// Adds argument amount to player health. This function will keep health greater or equal to zero.
    /// Note that there is no overflow checking.
    pub fn update_health(&mut self, amount: i32) {
        self.health += amount;

        if self.health < 0 {
            self.health = 0;
        }

        self.health_update = true;
    }

    /// Get current health if there is an health update occurred.
    pub fn health(&mut self) -> Option<u32> {
        if self.health_update {
            self.health_update = false;
            Some(self.health as u32)
        } else {
            None
        }
    }

    /// Return true if player is visible.
    pub fn visible(&self) -> bool {
        self.visible
    }
}

impl_traits!(Player);


/// Laser game object for enemy and player.
pub struct Laser {
    data: Data<f32>,
    speed: f32,
    destroy: bool,
    color: LaserColor,
}

impl Laser {
    /// Create new `Laser`.
    fn new(position: Vector2<f32>, color: LaserColor) -> Laser {
        let size = 1.5;
        Laser {
            data: Data::new(position, 0.10 * size, 0.05 * size),
            speed: LASER_SPEED,
            destroy: false,
            color: color,
        }
    }

    /// Create new `Laser` with specific width and height.
    fn new_with_width_and_height(position: Vector2<f32>, color: LaserColor, width: f32, height: f32) -> Laser {
        Laser {
            data: Data::new(position, width, height),
            speed: LASER_SPEED,
            destroy: false,
            color,
        }
    }

    /// Move laser forward and set laser to be destroyed if laser is not on the screen.
    fn update(&mut self, logic_settings: &LogicSettings, current_time: &GameTimeManager) {
        let speed = self.speed * current_time.delta_time();
        self.forward(speed);

        let width = logic_settings.screen_width_half + 1.0;
        let height = 5.5;
        let area = Rectangle::new(-width, width, -height, height);

        if self.outside_allowed_area(&area) {
            self.destroy = true;
        }
    }

    /// Get laser's color.
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

/// TODO: Split Enemy struct to two separate enemies?.

/// Enemy game object and logic.
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
    laser_cannon_top_laser_bomb_shooting_turn: bool,
    laser_bombs: Vec<LaserBomb>,
    laser_bomb_timer: Timer,
    laser_bomb_enabled: bool,
    shield: Shield,
    laser_x_position_margin: f32,
}

impl Enemy {
    /// Create new `Enemy`.
    fn new() -> Enemy {
        Enemy {
            data: Data::new_square(Vector2::zero(), 0.0),
            speed: 0.0,
            lasers: Vec::with_capacity(50),
            laser_timer: Timer::new(),
            health: ENEMY_MAX_HEALTH,
            health_update: true,
            visible: true,
            enemy_type: EnemyType::Normal,
            laser_cannon_top: LaserCannon::new(),
            laser_cannon_bottom: LaserCannon::new(),
            laser_cannon_top_laser_bomb_shooting_turn: true,
            laser_bombs: Vec::with_capacity(5),
            laser_bomb_timer: Timer::new(),
            laser_bomb_enabled: true,
            shield: Shield::new(Vector2::zero()),
            laser_x_position_margin: 0.0,
        }
    }

    /// Resets enemy position and settings to specific level.
    fn reset(&mut self, logic_settings: &LogicSettings, level: u32, current_time: &GameTimeManager) {
        self.lasers.clear();
        self.health = ENEMY_MAX_HEALTH;
        self.health_update = true;

        self.laser_bomb_timer.reset(current_time.time());
        self.laser_timer.reset(current_time.time());
        self.visible = true;

        if level == 0 || level == 2 {
            self.enemy_type = EnemyType::Normal;
            self.laser_x_position_margin = -0.5;
            self.data = Data::new_square(vec2(logic_settings.screen_width_half - 2.5, 0.0), ENEMY_SQUARE_SIDE_LENGTH);
            self.speed = ENEMY_MOVEMENT_SPEED;
        } else {
            self.enemy_type = EnemyType::Shield;
            self.laser_x_position_margin = -0.7;
            self.data = Data::new_square(vec2(logic_settings.screen_width_half - 3.0, 0.0), ENEMY_SQUARE_SIDE_LENGTH + 0.6);
            self.speed = ENEMY_WITH_SHIELD_MOVEMENT_SPEED;
        }

        if level >= 2 {
            self.laser_bomb_enabled = true;
        } else {
            self.laser_bomb_enabled = false;
        }

        self.laser_cannon_top_laser_bomb_shooting_turn = true;

        self.laser_bombs.clear();

        self.laser_cannon_bottom.reset(vec2(self.data.position.x, self.data.position.y - LASER_CANNON_DISTANCE_FROM_ENEMY), self.enemy_type, current_time);
        self.laser_cannon_top.reset(vec2(self.data.position.x, self.data.position.y + LASER_CANNON_DISTANCE_FROM_ENEMY), self.enemy_type, current_time);
        self.shield.reset(self.data.position, self.enemy_type);
    }

    /// Update enemy logic.
    fn update(&mut self,
            player: &mut Player,
            logic_settings: &LogicSettings,
            sounds: &mut SoundEffectManager,
            index_buffer: &mut Vec<usize>,
            current_time: &GameTimeManager) {
        // Enemy movement.
        let speed = self.speed;

        self.move_position(0.0, speed*current_time.delta_time());

        // Change enemy movement direction if enemy hits its movement borders.

        let width = logic_settings.screen_width_half - ENEMY_SQUARE_SIDE_LENGTH_HALF;
        let height = if let EnemyType::Shield = self.enemy_type {
            1.0
        } else {
            4.0
        };

        let area = Rectangle::new(-width, width, -height, height - GUI_MARGIN_TOP);

        if self.stay_at_area(&area) {
            self.speed *= -1.0;
        }

        // Enemy basic laser shooting.

        if self.laser_timer.check(current_time.time(), logic_settings.enemy_shooting_speed_milliseconds) {
            if let EnemyType::Shield = self.enemy_type {
                self.create_laser(consts::PI);
                self.create_laser(consts::PI * 0.9);
                self.create_laser(consts::PI * 1.1);
            } else {
                self.create_laser(consts::PI);
                if self.health < 20 {
                    self.create_laser(consts::PI * 0.9);
                    self.create_laser(consts::PI * 1.1);
                } else if self.health < 40 {
                    self.create_laser(consts::PI * 0.9);
                }
            }
        }

        // Updates enemy's normal lasers (non laser bomb lasers)

        self.lasers.update(index_buffer, &mut |laser| {
            laser.update(logic_settings, current_time);

            if laser.destroy() {
                true
            } else if player.circle_collision(laser) {
                player.update_health(-logic_settings.enemy_laser_damage);
                true
            } else {
                false
            }
        });


        if self.laser_bomb_enabled {
            // Update laser bombs.

            {
                let lasers = &mut self.lasers;
                self.laser_bombs.update(index_buffer, &mut |laser_bomb| {
                    laser_bomb.update(current_time, logic_settings, lasers, sounds);

                    if laser_bomb.destroy() {
                        true
                    } else if player.circle_collision(laser_bomb) {
                        player.update_health(-LASER_BOMB_DAMAGE);
                        true
                    } else {
                        false
                    }
                });
            }

            // Create new laser bomb.

            let laser_bomb_milliseconds = if logic_settings.difficulty == Difficulty::Hard && self.health <= 20 {
                ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS_HEALTH_20
            } else if (logic_settings.difficulty == Difficulty::Hard || logic_settings.difficulty == Difficulty::Normal) && self.health <= 40 {
                ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS_HEALTH_40
            } else {
                ENEMY_MILLISECONDS_BETWEEN_LASER_BOMBS_BEGINNING
            };

            if self.laser_bomb_timer.check(current_time.time(), laser_bomb_milliseconds) {
                sounds.laser_bomb_launch();
                self.create_laser_bomb(current_time);
            }
        }

        // EnemyType::Shield specific codes.

        if let EnemyType::Shield = self.enemy_type {
            // Shield enabling.

            if self.shield.update(self.data.position.y, current_time) {
                self.laser_cannon_top.parent_object_shield_enabled = true;
                self.laser_cannon_top.red_light = false;

                self.laser_cannon_bottom.parent_object_shield_enabled = true;
                self.laser_cannon_bottom.red_light = false;
            }

            // Shield disabling.

            if self.shield.visible && !self.laser_cannon_top.parent_object_shield_enabled && !self.laser_cannon_bottom.parent_object_shield_enabled {
                self.shield.disable(current_time);
            }

            // Enable laser cannon laser shooting depending on current enemy health.

            if self.health < 30 {
                self.laser_cannon_bottom.laser_enabled = true;
                self.laser_cannon_top.laser_enabled = true;
            } else if self.health < 60 {
                self.laser_cannon_top.laser_enabled = true;
            }

            // Update laser cannons.

            let y = self.y();
            self.laser_cannon_bottom.update(y - LASER_CANNON_DISTANCE_FROM_ENEMY, current_time, &mut self.lasers);
            self.laser_cannon_top.update(y + LASER_CANNON_DISTANCE_FROM_ENEMY, current_time, &mut self.lasers);
        }
    }

    /// Get enemy lasers.
    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    /// Creates new enemy laser. Laser game object will be turned
    /// with value given as argument turn_angle. This value must be in radians.
    fn create_laser(&mut self, turn_angle: f32) {
        let position = vec2(self.x() + self.laser_x_position_margin, self.y());
        let mut laser = Laser::new(position, LaserColor::Red);
        laser.turn(turn_angle);
        self.lasers.push(laser);
    }

    /// Creates new laser bomb. Laser bomb creation location will vary
    /// depending on current enemy type.
    fn create_laser_bomb(&mut self, current_time: &GameTimeManager) {
        let mut laser_bomb = match self.enemy_type {
            EnemyType::Normal => LaserBomb::new(vec2(self.x() + self.laser_x_position_margin, self.y()), current_time),
            EnemyType::Shield => {
                if self.laser_cannon_top_laser_bomb_shooting_turn {
                    self.laser_cannon_top_laser_bomb_shooting_turn = false;
                    let position = vec2(self.laser_cannon_top.x() - 0.5, self.laser_cannon_top.y());
                    LaserBomb::new(position, current_time)
                } else {
                    self.laser_cannon_top_laser_bomb_shooting_turn = true;
                    let position = vec2(self.laser_cannon_bottom.x() - 0.5, self.laser_cannon_bottom.y());
                    LaserBomb::new(position, current_time)
                }
            },
        };

        laser_bomb.turn(consts::PI);
        self.laser_bombs.push(laser_bomb);
    }

    /// Updates enemy health like player's health.
    /// See `Player` documentation for more details.
    pub fn update_health(&mut self, amount: i32) {
        self.health += amount;

        if self.health < 0 {
            self.health = 0;
        }

        self.health_update = true;
    }

    /// Get enemy health like player's health.
    /// See `Player` documentation for more details.
    pub fn health(&mut self) -> Option<u32> {
        if self.health_update {
            self.health_update = false;
            Some(self.health as u32)
        } else {
            None
        }
    }

    /// Return true if enemy is visible.
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Get laser cannon positioned at top.
    pub fn get_laser_cannon_top(&self) -> &LaserCannon {
        &self.laser_cannon_top
    }

    /// Get laser cannon positioned at bottom.
    pub fn get_laser_cannon_bottom(&self) -> &LaserCannon {
        &self.laser_cannon_bottom
    }

    /// Get laser bombs.
    pub fn get_laser_bombs(&self) -> &Vec<LaserBomb> {
        &self.laser_bombs
    }

    /// Get shield.
    pub fn get_shield(&self) -> &Shield {
        &self.shield
    }
}

impl_traits!(Enemy);


/// Shield protecting the enemy.
pub struct Shield {
    data: Data<f32>,
    visible: bool,
    timer: Timer,
}

impl Shield {
    /// Create new `Shield`.
    fn new(position: Vector2<f32>) -> Shield {
        let size = 2.25;
        Shield {
            data: Data::new_square(position, size),
            visible: false,
            timer: Timer::new(),
        }
    }

    /// Reset shield position and visibility depending on current enemy type.
    fn reset(&mut self, parent_position: Vector2<f32>, enemy_type: EnemyType) {
        if let EnemyType::Shield = enemy_type {
            self.visible = true;
        } else {
            self.visible = false;
            return;
        }

        self.set_position(parent_position.x, parent_position.y);
    }

    /// Updates shield position to match parent position. Check if shield should be enabled.
    /// Return true if shield is enabled during this update method call.
    fn update(&mut self, parent_position_y: f32, current_time: &GameTimeManager) -> bool {
        self.set_position_y(parent_position_y);

        if !self.visible && self.timer.check(current_time.time(), 10_000) {
            self.visible = true;
            true
        } else {
            false
        }
    }

    /// Return true if shield is visible.
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Disables shield.
    pub fn disable(&mut self, current_time: &GameTimeManager) {
        self.timer.reset(current_time.time());
        self.visible = false;
    }
}

impl_traits!(Shield);

/// Laser cannon logic.
pub struct LaserCannon {
    data: Data<f32>,
    visible: bool,
    laser_timer: Timer,
    parent_object_shield_enabled: bool,
    laser_enabled: bool,
    light_color_toggle_timer: Timer,
    red_light: bool,
}

impl LaserCannon {
    /// Create new `LaserCannon`.
    fn new() -> LaserCannon {
        let size = 0.7;

        LaserCannon {
            data: Data::new_square(Vector2::zero(), size),
            visible: false,
            laser_timer: Timer::new(),
            parent_object_shield_enabled: true,
            laser_enabled: false,
            light_color_toggle_timer: Timer::new(),
            red_light: false,
        }
    }

    /// Reset laser cannon state.
    fn reset(&mut self, new_position: Vector2<f32>, enemy_type: EnemyType, current_time: &GameTimeManager) {
        if let EnemyType::Shield = enemy_type {
            self.visible = true;
        } else {
            self.visible = false;
            return;
        }

        self.set_position(new_position.x, new_position.y);

        self.laser_timer.reset(current_time.time());
        self.light_color_toggle_timer.reset(current_time.time());

        self.parent_object_shield_enabled = true;
        self.laser_enabled = false;
        self.red_light = false;
    }

    /// Update laser cannon position and create lasers if lasers are enabled. Also updates laser cannon
    /// light animation.
    fn update(&mut self, new_y_position: f32, current_time: &GameTimeManager, parents_lasers: &mut Vec<Laser>) {
        if !self.visible {
            return;
        }

        if self.laser_enabled && self.laser_timer.check(current_time.time(), 1000) {
            let position = vec2(self.x() - 0.5, self.y());
            let mut laser = Laser::new(position, LaserColor::Red);
            laser.turn(consts::PI);
            parents_lasers.push(laser);
        }

        if !self.parent_object_shield_enabled && self.light_color_toggle_timer.check(current_time.time(), 400) {
            self.red_light = !self.red_light;
        }

        self.set_position_y(new_y_position);
    }

    /// Return true if laser cannon is visible.
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Return true if laser cannon's red light is enabled.
    pub fn red_light(&self) -> bool {
        self.red_light
    }
}

impl_traits!(LaserCannon);

/// Laser bomb creates lasers when it explodes.
/// This type is wrapper around `Laser` game object.
pub struct LaserBomb {
    laser: Laser,
    timer: Timer,
}

impl LaserBomb {
    /// Create new `LaserBomb`.
    fn new(position: Vector2<f32>, current_time: &GameTimeManager) -> LaserBomb {
        let size = 0.25;
        LaserBomb {
            laser: Laser::new_with_width_and_height(position, LaserColor::Blue, size, size),
            timer: Timer::new_from_time(current_time.time()),
        }
    }

    /// Updates laser logic and if there is enough time from laser bomb creation,
    /// the laser bomb will explode and create some lasers.
    fn update(&mut self, current_time: &GameTimeManager, logic_settings: &LogicSettings, parent_lasers: &mut Vec<Laser>, sounds: &mut SoundEffectManager) {
        self.laser.update(logic_settings, current_time);

        if self.timer.check(current_time.time(), LASER_BOMB_EXPLOSION_TIME_MILLISECONDS) {
            sounds.laser_bomb_explosion();
            let laser_count : u16 = 15;
            let mut angle = 0.0;
            let angle_between_lasers = (consts::PI*2.0) / f32::from(laser_count);

            for _ in 0..laser_count {
                let position = vec2(self.x(), self.y());
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

/// Background image that moves and resets it's position.
pub struct Background {
    data: Data<f32>,
    x_limit: f32,
    x_reset_position: f32,
    speed: f32,
}

impl Background {
    /// Create new `Background`.
    ///
    /// Background will be positioned at specific index in an "array" of backgrounds.
    /// Index zero is coordinate (0,0). Coordinate y stays always at zero.
    /// Backgrounds will not overlap because of the side length argument.
    ///
    /// Note that because of the current movement limits, the index must be at bounds of [-1; 2]
    fn new(i: i32, side_length: f32) -> Background {
        Background {
            data: Data::new_square(vec2(i as f32 * side_length, 0.0), side_length),
            x_limit: -2.0*side_length,
            x_reset_position: 2.0*side_length,
            speed: BACKGROUND_MOVING_SPEED,
        }
    }

    /// Moves background forward, and resets background position if it's x coordinate.
    /// goes under the current limit.
    fn update(&mut self, current_time: &GameTimeManager) {
        let speed = self.speed;
        self.move_position(speed*current_time.delta_time(), 0.0);

        if self.x() <= self.x_limit {
            self.data_mut().position.x = self.x_reset_position;
        }
    }
}


impl_traits!(Background);

/// Many moving backgrounds in an array to make and "infinite" background.
pub struct MovingBackground {
    backgrounds: [Background; 4],
}

impl MovingBackground {
    /// Create new `MovingBackground`
    pub fn new() -> MovingBackground {
        let size = BACKGROUND_SQUARE_SIDE_LENGTH;
        let backgrounds = [
            Background::new(-1, size),
            Background::new(0, size),
            Background::new(1, size),
            Background::new(2, size),
        ];

        MovingBackground { backgrounds }
    }

    /// Updates all backgrounds.
    fn update(&mut self, current_time: &GameTimeManager) {
       for background in &mut self.backgrounds {
           background.update(current_time);
       }
    }

    /// Get backgrounds.
    pub fn get_backgrounds(&self) -> &[Background; 4] {
        &self.backgrounds
    }

    /// Moves every background's x coordinate.
    pub fn move_position_x(&mut self, x: f32) {
        for background in self.backgrounds.iter_mut() {
            background.move_position(x, 0.0);
        }
    }
}