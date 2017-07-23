/*
src/logic/mod.rs, 2017-07-23

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


pub struct Logic {
    player: Player,
    enemy: Enemy,
}

impl Logic {
    pub fn new() -> Logic {
        let player = Player::new();
        let enemy = Enemy::new();
        Logic { player, enemy }
    }

    pub fn update<T: Input>(&mut self, input: &T) {
        self.player.update(input, &mut self.enemy);
        self.enemy.update(&mut self.player);
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }

    pub fn get_enemy(&self) -> &Enemy {
        &self.enemy
    }
}

pub struct Player {
    data: Data<f32>,
    speed: f32,
    lasers: Vec<Laser>,
    laser_timer: Timer,
    health: i32,
}

impl Player {
    fn new() -> Player {
        let data = Data::new(0.0, 0.0, 1.0, 1.0);
        let speed = 0.05;
        let lasers = vec![];
        let laser_timer = Timer::new();
        let health = 100;
        Player { data, speed, lasers, laser_timer, health }
    }

    fn update(&mut self, input: &Input, enemy: &mut Enemy) {
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
            let laser = Laser::new(self.data().position.x + 1.0, self.data().position.y);
            self.lasers.push(laser);
        }

        let width = 5.0;
        let height = 4.0;
        let area = Rectangle::new(-width, width, -height, height);
        self.stay_at_area(&area);

        self.clean_and_update_lasers(enemy);

        if self.circle_collision(enemy) {
            self.update_health(-1);
        }
    }

    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self, enemy: &mut Enemy) {
        let mut remove = (false, 0);

        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.update();

            if laser.destroy() {
                remove = (true, i);
            } else if enemy.circle_collision(laser) {
                remove = (true, i);
                enemy.update_health(-laser.get_damage());
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
    }
}


impl GameObject for Player {}
impl ModelMatrix for Player {}


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
    damage: i32,
}

impl Laser {
    fn new(x: f32, y: f32) -> Laser {
        let data = Data::new(x, y, 0.3, 0.1);
        let speed = 0.08;
        let destroy = false;
        let damage = 1;
        Laser { data, speed, destroy, damage }
    }

    fn update(&mut self) {
        let speed = self.speed;
        self.forward(speed);

        let width = 5.0;
        let height = 4.0;
        let area = Rectangle::new(-width, width, -height, height);

        if self.outside_allowed_area(&area) {
            self.destroy = true;
        }
    }

    fn get_damage(&self) -> i32 {
        self.damage
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
impl ModelMatrix for Laser {}


pub struct Enemy {
    data: Data<f32>,
    speed: f32,
    lasers: Vec<Laser>,
    laser_timer: Timer,
    health: i32,
}

impl Enemy {
    fn new() -> Enemy {
        let data = Data::new(3.0, 0.0, 1.0, 1.0);

        let speed = 0.05;
        let lasers = vec![];
        let laser_timer = Timer::new();
        let health = 100;
        Enemy { data, speed, lasers, laser_timer, health }
    }

    fn update(&mut self, player: &mut Player) {
        let speed = self.speed;

        self.move_position(0.0, speed);

        let width = 5.0;
        let height = 4.0;
        let area = Rectangle::new(-width, width, -height, height);

        if self.stay_at_area(&area) {
            self.speed *= -1.0;
        }

        if self.laser_timer.check(PreciseTime::now(), 1000) {
            self.create_laser(consts::PI);
            if (self.health < 25) {
                self.create_laser(consts::PI * 0.9);
                self.create_laser(consts::PI * 1.1);
            } else if (self.health < 50) {
                self.create_laser(consts::PI * 0.9);
            }
        }

        self.clean_and_update_lasers(player);
    }

    pub fn get_lasers(&self) -> &Vec<Laser> {
        &self.lasers
    }

    fn clean_and_update_lasers(&mut self, player: &mut Player) {
        let mut remove = (false, 0);

        for (i, laser) in self.lasers.iter_mut().enumerate() {
            laser.update();

            if laser.destroy() {
                remove = (true, i);
            } else if player.circle_collision(laser) {
                remove = (true, i);
                player.update_health(-laser.get_damage());
            }
        }

        if let (true, i) = remove {
            self.lasers.swap_remove(i);
        }
    }

    fn create_laser(&mut self, rotation: f32) {
        let mut laser = Laser::new(self.data().position.x - 1.0, self.data().position.y);
        laser.turn(rotation);
        self.lasers.push(laser);
    }

    pub fn update_health(&mut self, amount: i32) {
        self.health += amount;

        if self.health < 0 {
            self.health = 0;
        }

        println!("enemy health: {}", self.health);
    }
}


impl GameObject for Enemy {}
impl ModelMatrix for Enemy {}


impl GameObjectData<f32> for Enemy {
    fn data(&self) -> &Data<f32> {
        &self.data
    }
    fn data_mut(&mut self) -> &mut Data<f32> {
        &mut self.data
    }
}
