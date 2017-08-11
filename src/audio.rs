/*
src/audio.rs, 2017-08-11

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

use sdl2::mixer::{Sdl2MixerContext, Channel, Chunk, InitFlag, Music};
use sdl2::mixer;

use sdl2::AudioSubsystem;

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
    fn new(_mixer_context: &Sdl2MixerContext, file_path: &str) -> SoundEffect {
        SoundEffect {
            channel: Channel::all(),
            chunk: Chunk::from_file(file_path).expect("error"),
        }
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

pub struct SoundEffectManager {
    laser: SoundEffect,
    explosion: SoundEffect,
    laser_bomb_launch: SoundEffect,
    laser_bomb_explosion: SoundEffect,
    player_laser_hits_laser_cannon: SoundEffect,
    effect_volume: i32,
}

impl SoundEffectManager {
    fn new(mixer_context: &Sdl2MixerContext) -> SoundEffectManager {
        let default_volume = mixer::MAX_VOLUME;

        let mut sounds = SoundEffectManager {
            laser:                  SoundEffect::new(mixer_context, "game_files/audio/laser.wav"),
            explosion:              SoundEffect::new(mixer_context, "game_files/audio/explosion.wav"),
            laser_bomb_launch:      SoundEffect::new(mixer_context, "game_files/audio/laser_bomb_launch.wav"),
            laser_bomb_explosion:   SoundEffect::new(mixer_context, "game_files/audio/laser_bomb_explosion.wav"),
            player_laser_hits_laser_cannon:   SoundEffect::new(mixer_context, "game_files/audio/player_laser_hits_laser_cannon.wav"),
            effect_volume: default_volume,
        };

        sounds.change_volume(default_volume);

        sounds
    }

    fn all_mut(&mut self) -> [&mut SoundEffect; 4] {
        [
            &mut self.laser,
            &mut self.explosion,
            &mut self.laser_bomb_launch,
            &mut self.laser_bomb_explosion,
        ]
    }

    pub fn change_volume(&mut self, volume: i32) {
        let volume = check_volume_value(volume);

        self.effect_volume = volume;

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

impl SoundEffectPlayer for SoundEffectManager {
    fn laser(&mut self) {
        self.laser.play_if_not_playing();
    }

    fn laser_bomb_launch(&mut self) {
        self.laser_bomb_launch.play();
    }

    fn laser_bomb_explosion(&mut self) {
        self.laser_bomb_explosion.play();
    }

    fn explosion(&mut self) {
        self.explosion.play();
    }

    fn player_laser_hits_laser_cannon(&mut self) {
        self.player_laser_hits_laser_cannon.play();
    }
}


pub struct AudioManager {
    audio_subsystem: AudioSubsystem,
    _mixer_context: Sdl2MixerContext,
    sound_effects: SoundEffectManager,
    music: Option<Music<'static>>,
    music_volume: i32,
}

impl AudioManager {
    pub fn new(audio_subsystem: AudioSubsystem) -> AudioManager {
        let mut music_support = true;

        println!("");

        let _mixer_context = match mixer::init(mixer::INIT_OGG) {
            Ok(context) => context,
            Err(error) => {
                println!("SDL_mixer init error: {}", error);
                println!("trying to init SDL_mixer without music support");

                music_support = false;

                mixer::init(InitFlag::empty()).expect("SDL_mixer init error")
            }
        };

        if !music_support {
            println!("game music disabled");
        }

        println!("SDL_mixer version: {}", mixer::get_linked_version());

        mixer::open_audio(mixer::DEFAULT_FREQUENCY, mixer::DEFAULT_FORMAT, mixer::DEFAULT_CHANNELS, 1024).expect("error");

        let sound_effects = SoundEffectManager::new(&_mixer_context);

        let music_default_volume = mixer::MAX_VOLUME;
        Music::set_volume(music_default_volume);

        let music = if music_support {
            match Music::from_file("game_files/audio/music.ogg") {
                Ok(music) => Some(music),
                Err(error) => {
                    println!("music loading error: {}", error);
                    None
                }
            }
        } else {
            None
        };

        println!("SDL2 current audio driver: {}", audio_subsystem.current_audio_driver());

        if let Some(number) = audio_subsystem.num_audio_playback_devices() {
            for i in 0..number {
                println!("playback device index: {}, name: {}", i, audio_subsystem.audio_playback_device_name(i).expect("error"));
            }
        }

        AudioManager {
            audio_subsystem,
            _mixer_context,
            sound_effects,
            music,
            music_volume: music_default_volume,
        }
    }

    pub fn sound_effect_manager_mut(&mut self) -> &mut SoundEffectManager {
        &mut self.sound_effects
    }

    pub fn set_music_volume(&mut self, volume: i32) {
        let volume = check_volume_value(volume);
        self.music_volume = volume;

        Music::set_volume(volume);
    }

    pub fn max_volume() -> i32 {
        mixer::MAX_VOLUME
    }

    pub fn play_music(&mut self) {
        if let Some(ref music) = self.music {
            music.play(-1).expect("error");
        }
    }
}
