/*
src/audio.rs, 2017-09-02

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Play sound effects and music.

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

pub trait Audio: Sized {
    type Volume: Volume;

    fn load(&str) -> Result<Self, String>;
    fn play(&mut self);
    fn change_volume(&mut self, volume: Self::Volume);
}

pub trait Volume: Copy + Clone {
    type Value;
    const MAX_VOLUME: Self::Value;
    const DEFAULT_VOLUME_PERCENTAGE: i32;

    /// Create new volume value limited to [0; MAX_VOLUME].
    fn new(Self::Value) -> Self;
    fn value(&self) -> Self::Value;

    /// Create Volume value from integer representing
    /// volume percentage. Clamps integer to range [0; 100].
    fn from_percentage(i32) -> Self;
}

/// All sound effect's that the game requires.
struct AllSoundEffects<A: Audio> {
    laser: A,
    explosion: A,
    laser_bomb_launch: A,
    laser_bomb_explosion: A,
    player_laser_hits_laser_cannon: A,
}

impl <A: Audio> AllSoundEffects<A> {

    /// Loads all sound effects that the game requires.
    fn new(default_volume: A::Volume) -> Result<Self, String> {

        let mut sounds = AllSoundEffects {
            laser:                  A::load("game_files/audio/laser.wav")?,
            explosion:              A::load("game_files/audio/explosion.wav")?,
            laser_bomb_launch:      A::load("game_files/audio/laser_bomb_launch.wav")?,
            laser_bomb_explosion:   A::load("game_files/audio/laser_bomb_explosion.wav")?,
            player_laser_hits_laser_cannon:   A::load("game_files/audio/player_laser_hits_laser_cannon.wav")?,
        };

        sounds.change_volume(default_volume);

        Ok(sounds)
    }

    /// All sound effects as array of mutable references.
    fn all_mut(&mut self) -> [&mut A; 5] {
        [
            &mut self.laser,
            &mut self.explosion,
            &mut self.laser_bomb_launch,
            &mut self.laser_bomb_explosion,
            &mut self.player_laser_hits_laser_cannon,
        ]
    }

    /// Change volume of every sound effect.
    fn change_volume(&mut self, volume: A::Volume) {
        for effect in self.all_mut().iter_mut() {
            effect.change_volume(volume);
        }
    }
}



/// Stores sound effects and boolean values about
/// what sound effect should be played.
pub struct SoundEffectManager<A: Audio> {
    sound_effects: Option<AllSoundEffects<A>>,
    laser: bool,
    laser_bomb_launch: bool,
    laser_bomb_explosion: bool,
    explosion: bool,
    player_laser_hits_laser_cannon: bool,
}

impl <A: Audio> SoundEffectManager<A> {

    /// Create new `SoundEffectManager`.
    ///
    /// If argument `sound_effects` is `None`, sound effects are not played.
    fn new(sound_effects: Option<AllSoundEffects<A>>) -> Self {
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
    fn play(play_sound_effect: &mut bool, sound_effect: &mut A ) {
        if *play_sound_effect {
            *play_sound_effect = false;
            sound_effect.play();
        }
    }

    /// Change volume of all sound effects if sound effects are available.
    fn change_volume(&mut self, volume: A::Volume) {
        if let Some(ref mut effects) = self.sound_effects {
            effects.change_volume(volume);
        }
    }
}

impl <A: Audio> SoundEffectPlayer for SoundEffectManager<A> {
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

pub trait AudioPlayer {
    type Music: Audio;
    type Effect: Audio;
}

/// Store music, sound effects, volume values.
pub struct AudioManager<P: AudioPlayer> {
    _player: Option<P>,
    sound_effects: SoundEffectManager<P::Effect>,
    music: Option<P::Music>,
    music_volume: <P::Music as Audio>::Volume,
    effect_volume: <P::Effect as Audio>::Volume,
}

impl <P: AudioPlayer> AudioManager<P> {
    /// Create new `AudioManager`.
    ///
    /// Sound effects will be loaded from default locations. If there is
    /// sound effect loading error, all sound effects will be disabled.
    ///
    /// If there is a music loading error, music will be disabled.
    ///
    /// If argument player is `None`, sound effects and music will be disabled.
    ///
    /// All errors will be printed to standard output.
    pub fn new(music_file_path: &str, player: Option<P>) -> Self {
        println!("");

        let music_volume = <P::Music as Audio>::Volume::from_percentage(<P::Music as Audio>::Volume::DEFAULT_VOLUME_PERCENTAGE);
        let effect_volume = <P::Effect as Audio>::Volume::from_percentage(<P::Effect as Audio>::Volume::DEFAULT_VOLUME_PERCENTAGE);

        match player {
            Some(_) => {
                let music = match P::Music::load(music_file_path) {
                    Ok(mut music) => {
                        music.change_volume(music_volume);
                        Some(music)
                    }
                    Err(error) => {
                        println!("music loading error: {}", error);
                        None
                    }
                };

                let all_sound_effects = match AllSoundEffects::new(effect_volume) {
                    Ok(sound_effects) => Some(sound_effects),
                    Err(error) => {
                        println!("error when loading sound effects: {}", error);
                        None
                    },
                };

                Self {
                    sound_effects: SoundEffectManager::new(all_sound_effects),
                    music,
                    music_volume,
                    effect_volume,
                    _player: player,
                }
            }
            None => {
                println!("Audio support disabled");

                Self {
                    sound_effects: SoundEffectManager::new(None),
                    music: None,
                    music_volume,
                    effect_volume,
                    _player: None,
                }
            }
        }
    }

    /// Get `SoundEffectManager`
    pub fn sound_effect_manager_mut(&mut self) -> &mut SoundEffectManager<P::Effect> {
        &mut self.sound_effects
    }

    /// Set music volume.
    pub fn set_music_volume(&mut self, volume_percentage: i32) {
        self.music_volume = <P::Music as Audio>::Volume::from_percentage(volume_percentage);

        if let Some(ref mut music) = self.music {
            music.change_volume(self.music_volume);
        }
    }

    /// Start playing music.
    pub fn play_music(&mut self) {
        if let Some(ref mut music) = self.music {
            music.play();
        }
    }

    /// Set sound effect volume.
    pub fn set_sound_effect_volume(&mut self, volume_percentage: i32) {
        self.effect_volume = <P::Effect as Audio>::Volume::from_percentage(volume_percentage);

        self.sound_effects.change_volume(self.effect_volume);
    }
}
