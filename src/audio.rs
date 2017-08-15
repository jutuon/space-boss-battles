/*
src/audio.rs, 2017-08-15

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use sdl2::mixer::{Channel, Chunk, Music};
use sdl2::mixer;

pub const MAX_VOLUME: i32 = mixer::MAX_VOLUME;
pub const DEFAULT_VOLUME: i32 = mixer::MAX_VOLUME;

pub trait SoundEffectPlayer {
    fn laser(&mut self);
    fn laser_bomb_launch(&mut self);
    fn laser_bomb_explosion(&mut self);
    fn explosion(&mut self);
    fn player_laser_hits_laser_cannon(&mut self);
}

struct SoundEffect {
    channel: Channel,
    chunk: Chunk,
}

impl SoundEffect {
    fn new(file_path: &str) -> Result<SoundEffect, String> {
        let sound_effect = SoundEffect {
            channel: Channel::all(),
            chunk: Chunk::from_file(file_path)?,
        };

        Ok(sound_effect)
    }

    fn play(&mut self) {
        self.channel = match self.channel.play(&self.chunk, 0) {
            Ok(channel) => channel,
            Err(message) => {
                println!("sound effect playing error: {}", message);
                Channel::all()
            },
        };
    }

    fn play_if_not_playing(&mut self) {
        if !self.channel.is_playing() {
            self.play();
        }
    }

    fn change_volume(&mut self, volume: i32) {
        self.chunk.set_volume(volume);
    }
}

struct AllSoundEffects {
    laser: SoundEffect,
    explosion: SoundEffect,
    laser_bomb_launch: SoundEffect,
    laser_bomb_explosion: SoundEffect,
    player_laser_hits_laser_cannon: SoundEffect,
}

impl AllSoundEffects {
    fn new(default_volume: i32) -> Result<AllSoundEffects, String> {

        let mut sounds = AllSoundEffects {
            laser:                  SoundEffect::new("game_files/audio/laser.wav")?,
            explosion:              SoundEffect::new("game_files/audio/explosion.wav")?,
            laser_bomb_launch:      SoundEffect::new("game_files/audio/laser_bomb_launch.wav")?,
            laser_bomb_explosion:   SoundEffect::new("game_files/audio/laser_bomb_explosion.wav")?,
            player_laser_hits_laser_cannon:   SoundEffect::new("game_files/audio/player_laser_hits_laser_cannon.wav")?,
        };

        sounds.change_volume(default_volume);

        Ok(sounds)
    }

    fn all_mut(&mut self) -> [&mut SoundEffect; 5] {
        [
            &mut self.laser,
            &mut self.explosion,
            &mut self.laser_bomb_launch,
            &mut self.laser_bomb_explosion,
            &mut self.player_laser_hits_laser_cannon,
        ]
    }

    fn change_volume(&mut self, volume: i32) {
        for effect in self.all_mut().iter_mut() {
            effect.change_volume(volume);
        }
    }
}

pub fn check_volume_value(volume: i32) -> i32 {
    if volume > mixer::MAX_VOLUME {
        mixer::MAX_VOLUME
    } else if volume < 0 {
        0
    } else {
        volume
    }
}


pub struct SoundEffectManager {
    sound_effects: Option<AllSoundEffects>,
    laser: bool,
    laser_bomb_launch: bool,
    laser_bomb_explosion: bool,
    explosion: bool,
    player_laser_hits_laser_cannon: bool,
}

impl SoundEffectManager {
    fn new(sound_effects: Option<AllSoundEffects>) -> SoundEffectManager {
        SoundEffectManager {
            sound_effects,
            laser: false,
            laser_bomb_launch: false,
            laser_bomb_explosion: false,
            explosion: false,
            player_laser_hits_laser_cannon: false,
        }
    }

    pub fn update(&mut self) {
        if let Some(ref mut effects) = self.sound_effects {
            SoundEffectManager::play_if_not_playing(&mut self.laser, &mut effects.laser);
            SoundEffectManager::play(&mut self.laser_bomb_launch, &mut effects.laser_bomb_launch);
            SoundEffectManager::play(&mut self.laser_bomb_explosion, &mut effects.laser_bomb_explosion);
            SoundEffectManager::play(&mut self.explosion, &mut effects.explosion);
            SoundEffectManager::play(&mut self.player_laser_hits_laser_cannon, &mut effects.player_laser_hits_laser_cannon);
        }
    }

    fn play_if_not_playing(play_sound_effect: &mut bool, sound_effect: &mut SoundEffect ) {
        if *play_sound_effect {
            *play_sound_effect = false;
            sound_effect.play_if_not_playing();
        }
    }

    fn play(play_sound_effect: &mut bool, sound_effect: &mut SoundEffect ) {
        if *play_sound_effect {
            *play_sound_effect = false;
            sound_effect.play();
        }
    }

    fn change_volume(&mut self, volume: i32) {
        if let Some(ref mut effects) = self.sound_effects {
            effects.change_volume(volume);
        }
    }
}

impl SoundEffectPlayer for SoundEffectManager {
    fn laser(&mut self) {
        self.laser = true;
    }

    fn laser_bomb_launch(&mut self) {
        self.laser_bomb_launch = true;
    }

    fn laser_bomb_explosion(&mut self) {
        self.laser_bomb_explosion = true;
    }

    fn explosion(&mut self) {
        self.explosion = true;
    }

    fn player_laser_hits_laser_cannon(&mut self) {
        self.player_laser_hits_laser_cannon = true;
    }
}

struct MusicWrapper {
    music: Music<'static>,
}

impl MusicWrapper {
    fn new(music_file_path: &str) -> Result<MusicWrapper, String> {
        let music_wrapper = MusicWrapper {
            music: Music::from_file(music_file_path)?,
        };

        Ok( music_wrapper )
    }

    fn set_volume(&mut self, volume: i32) {
        Music::set_volume(volume);
    }

    fn play(&mut self) {
        if !Music::is_playing() {
            if let Err(message) = self.music.play(-1) {
                println!("music error: {}", message);
            }
        }
    }
}


pub struct AudioManager {
    sound_effects: SoundEffectManager,
    music: Option<MusicWrapper>,
    music_volume: i32,
    sound_effect_volume: i32,
    audio_open: bool,
}

impl AudioManager {
    pub fn new(music_file_path: &str) -> AudioManager {
        println!("");

        if let Err(error) = mixer::open_audio(mixer::DEFAULT_FREQUENCY, mixer::DEFAULT_FORMAT, mixer::DEFAULT_CHANNELS, 1024) {
            println!("SDL_mixer init error: {}", error);
            println!("Audio support disabled");

            AudioManager {
                sound_effects: SoundEffectManager::new(None),
                music: None,
                music_volume: DEFAULT_VOLUME,
                sound_effect_volume: DEFAULT_VOLUME,
                audio_open: false,
            }
        } else {
            println!("SDL_mixer version: {}", mixer::get_linked_version());

            let music = match MusicWrapper::new(music_file_path) {
                Ok(mut music) => {
                    music.set_volume(DEFAULT_VOLUME);
                    Some(music)
                }
                Err(error) => {
                    println!("music loading error: {}", error);
                    None
                }
            };

            let all_sound_effects = match AllSoundEffects::new(DEFAULT_VOLUME) {
                Ok(sound_effects) => Some(sound_effects),
                Err(error) => {
                    println!("error when loading sound effects: {}", error);
                    None
                },
            };

            AudioManager {
                sound_effects: SoundEffectManager::new(all_sound_effects),
                music,
                music_volume: DEFAULT_VOLUME,
                sound_effect_volume: DEFAULT_VOLUME,
                audio_open: true,
            }
        }
    }

    pub fn sound_effect_manager_mut(&mut self) -> &mut SoundEffectManager {
        &mut self.sound_effects
    }

    pub fn set_music_volume(&mut self, volume: i32) {
        let volume = check_volume_value(volume);
        self.music_volume = volume;

        if let Some(ref mut music) = self.music {
            music.set_volume(volume);
        }
    }

    pub fn play_music(&mut self) {
        if let Some(ref mut music) = self.music {
            music.play();
        }
    }

    pub fn set_sound_effect_volume(&mut self, volume: i32) {
        let volume = check_volume_value(volume);

        self.sound_effect_volume = volume;

        self.sound_effects.change_volume(volume);
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        if self.audio_open {
            mixer::close_audio();
        }
    }
}
