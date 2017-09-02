/*
src/audio.rs, 2017-09-02

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Play sound effects and music.

use sdl2::mixer::{Channel, Chunk, Music};
use sdl2::mixer;

pub const MAX_VOLUME: i32 = mixer::MAX_VOLUME;
pub const DEFAULT_VOLUME: i32 = 88;

/// Play sound effects.
pub trait SoundEffectPlayer {
    /// Play laser sound at next update.
    fn laser(&mut self);
    /// Play laser bomb launch sound at next update.
    fn laser_bomb_launch(&mut self);
    /// Play laser bomb explosion sound at next update.
    fn laser_bomb_explosion(&mut self);
    /// Play explosion sound at next update.
    fn explosion(&mut self);
    /// Play player laser hits laser cannon sound at next update.
    fn player_laser_hits_laser_cannon(&mut self);
    /// Play sound effects that are set to be played if
    /// sound effects are available.
    fn update(&mut self);
}

/// Sound effect's audio data and current `sdl2::mixer::Channel`
struct SoundEffect {
    channel: Channel,
    chunk: Chunk,
}

impl SoundEffect {
    /// Load new sound effect.
    fn new(file_path: &str) -> Result<SoundEffect, String> {
        let sound_effect = SoundEffect {
            channel: Channel::all(),
            chunk: Chunk::from_file(file_path)?,
        };

        Ok(sound_effect)
    }

    /// Play sound effect.
    ///
    /// Prints error message to standard output if there is sound effect
    /// playing error.
    fn play(&mut self) {
        self.channel = match self.channel.play(&self.chunk, 0) {
            Ok(channel) => channel,
            Err(message) => {
                println!("sound effect playing error: {}", message);
                Channel::all()
            },
        };
    }

    /// Change sound effect's volume.
    fn change_volume(&mut self, volume: Volume) {
        self.chunk.set_volume(volume.0);
    }
}

/// All sound effect's that the game requires.
struct AllSoundEffects {
    laser: SoundEffect,
    explosion: SoundEffect,
    laser_bomb_launch: SoundEffect,
    laser_bomb_explosion: SoundEffect,
    player_laser_hits_laser_cannon: SoundEffect,
}

impl AllSoundEffects {

    /// Loads all sound effects that the game requires.
    fn new(default_volume: Volume) -> Result<AllSoundEffects, String> {

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

    /// All sound effects as array of mutable references.
    fn all_mut(&mut self) -> [&mut SoundEffect; 5] {
        [
            &mut self.laser,
            &mut self.explosion,
            &mut self.laser_bomb_launch,
            &mut self.laser_bomb_explosion,
            &mut self.player_laser_hits_laser_cannon,
        ]
    }

    /// Change volume of every sound effect.
    fn change_volume(&mut self, volume: Volume) {
        for effect in self.all_mut().iter_mut() {
            effect.change_volume(volume);
        }
    }
}

/// Wrapper for correct audio volume value.
#[derive(Copy, Clone)]
pub struct Volume(i32);

impl Volume {
    /// Create new volume value limited to [0; MAX_VOLUME].
    pub fn new(volume: i32) -> Volume {
        if volume > MAX_VOLUME {
            Volume(MAX_VOLUME)
        } else if volume < 0 {
            Volume(0)
        } else {
            Volume(volume)
        }
    }

    /// Get volume value.
    pub fn value(&self) -> i32 {
        self.0
    }
}



/// Stores sound effects and boolean values about
/// what sound effect should be played.
pub struct SoundEffectManager {
    sound_effects: Option<AllSoundEffects>,
    laser: bool,
    laser_bomb_launch: bool,
    laser_bomb_explosion: bool,
    explosion: bool,
    player_laser_hits_laser_cannon: bool,
}

impl SoundEffectManager {

    /// Create new `SoundEffectManager`.
    ///
    /// If argument `sound_effects` is `None`, sound effects are not played.
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

    /// Check if sound effect should be played and plays it with function `play`.
    ///
    /// Resets argument `play_sound_effect` to false if it was true.
    fn play(play_sound_effect: &mut bool, sound_effect: &mut SoundEffect ) {
        if *play_sound_effect {
            *play_sound_effect = false;
            sound_effect.play();
        }
    }

    /// Change volume of all sound effects if sound effects are available.
    fn change_volume(&mut self, volume: Volume) {
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

    fn update(&mut self) {
        if let Some(ref mut effects) = self.sound_effects {
            SoundEffectManager::play(&mut self.laser, &mut effects.laser);
            SoundEffectManager::play(&mut self.laser_bomb_launch, &mut effects.laser_bomb_launch);
            SoundEffectManager::play(&mut self.laser_bomb_explosion, &mut effects.laser_bomb_explosion);
            SoundEffectManager::play(&mut self.explosion, &mut effects.explosion);
            SoundEffectManager::play(&mut self.player_laser_hits_laser_cannon, &mut effects.player_laser_hits_laser_cannon);
        }
    }
}

/// Wrapper for `sdl2::mixer::Music`.
struct MusicWrapper {
    music: Music<'static>,
}

impl MusicWrapper {
    /// Load music from file.
    fn new(music_file_path: &str) -> Result<MusicWrapper, String> {
        let music_wrapper = MusicWrapper {
            music: Music::from_file(music_file_path)?,
        };

        Ok( music_wrapper )
    }

    /// Set music volume.
    fn set_volume(&mut self, volume: Volume) {
        Music::set_volume(volume.0);
    }

    /// Start playing music if it isn't already playing.
    ///
    /// If starting the music failed, an error message will
    /// be printed to the standard output.
    fn play(&mut self) {
        if !Music::is_playing() {
            if let Err(message) = self.music.play(-1) {
                println!("music error: {}", message);
            }
        }
    }
}

/// Store music, sound effects, volume values.
pub struct AudioManager {
    sound_effects: SoundEffectManager,
    music: Option<MusicWrapper>,
    music_volume: Volume,
    sound_effect_volume: Volume,
    audio_open: bool,
}

impl AudioManager {
    /// Create new `AudioManager`.
    ///
    /// Sound effects will be loaded from default locations. If there is
    /// sound effect loading error, all sound effects will be disabled.
    ///
    /// If there is a music loading error, music will be disabled.
    ///
    /// If there is a SDL_mixer init error, sound effects and music will be disabled.
    ///
    /// All errors will be printed to standard output.
    pub fn new(music_file_path: &str) -> AudioManager {
        println!("");

        let default_volume = Volume::new(DEFAULT_VOLUME);

        if let Err(error) = mixer::open_audio(44100, mixer::DEFAULT_FORMAT, mixer::DEFAULT_CHANNELS, 1024) {
            println!("SDL_mixer init error: {}", error);
            println!("Audio support disabled");

            AudioManager {
                sound_effects: SoundEffectManager::new(None),
                music: None,
                music_volume: default_volume,
                sound_effect_volume: default_volume,
                audio_open: false,
            }
        } else {
            println!("SDL_mixer version: {}", mixer::get_linked_version());

            let music = match MusicWrapper::new(music_file_path) {
                Ok(mut music) => {
                    music.set_volume(default_volume);
                    Some(music)
                }
                Err(error) => {
                    println!("music loading error: {}", error);
                    None
                }
            };

            let all_sound_effects = match AllSoundEffects::new(default_volume) {
                Ok(sound_effects) => Some(sound_effects),
                Err(error) => {
                    println!("error when loading sound effects: {}", error);
                    None
                },
            };

            AudioManager {
                sound_effects: SoundEffectManager::new(all_sound_effects),
                music,
                music_volume: default_volume,
                sound_effect_volume: default_volume,
                audio_open: true,
            }
        }
    }

    /// Get `SoundEffectManager`
    pub fn sound_effect_manager_mut(&mut self) -> &mut SoundEffectManager {
        &mut self.sound_effects
    }

    /// Set music volume.
    pub fn set_music_volume(&mut self, volume: Volume) {
        self.music_volume = volume;

        if let Some(ref mut music) = self.music {
            music.set_volume(volume);
        }
    }

    /// Start playing music.
    pub fn play_music(&mut self) {
        if let Some(ref mut music) = self.music {
            music.play();
        }
    }

    /// Set sound effect volume.
    pub fn set_sound_effect_volume(&mut self, volume: Volume) {
        self.sound_effect_volume = volume;

        self.sound_effects.change_volume(volume);
    }
}

impl Drop for AudioManager {
    /// Call function `sdl2::mixer::close_audio` if audio is opened.
    fn drop(&mut self) {
        if self.audio_open {
            mixer::close_audio();
        }
    }
}
