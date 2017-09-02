# Space Boss Battles

Space Boss Battles is 2D arcade space shooter game. This version of the game is complete rewrite in Rust programming language. Originally game was written in CoolBasic programming language.

This project is also my first proper project written in Rust and will also serve as coursework for programming project course.

For more background information about the project and it's current architecture, see [documentation/project-self-evaluation.md](/documentation/project-self-evaluation.md).

Downloadable builds are not currently available so compiling source code is required to play the game or you
can test the [experimental web browser build](https://jutuon.github.io/space-boss-battles/).

## Features

Game:

- [x] 3 difficulty settings
- [x] 4 game levels

Graphical User Interface, GUI:

- [x] Main menu
- [x] Settings
- [x] Pause menu
- [x] Display game status
- [x] FPS counter
- [ ] GUI scaling for different display sizes

Input:

- [x] Keyboard
- [x] Mouse (only GUIButton support)
- [x] Game controllers
- [ ] Touch screen
- [ ] Configurable controls

Renderer:

- [x] OpenGL 3.3 and OpenGL ES 2.0 support
- [x] Full screen mode
- [x] Option to disable vertical synchronization

Audio:

- [x] Sound effects
- [ ] Music (not included, but supported)

Platforms:

- [x] Linux
- [ ] Windows
- [ ] Mac
- [ ] Android
- [x] Web (experimental support)
- [x] Raspberry Pi

General features:

- [x] Settings file

## User guide

### Controls

#### Keyboard

  Key                                                       |   Action
------------------------------------------------------------|-----------
<kbd>W</kbd><kbd>A</kbd><kbd>S</kbd><kbd>D</kbd> or <kbd>Up</kbd><kbd>Down</kbd><kbd>Left</kbd><kbd>Right</kbd>  | Move
<kbd>Space</kbd> or <kbd>LeftCtrl</kbd> or <kbd>RightCtrl</kbd>       | Shoot
<kbd>Esc</kbd>                 | Pause game
<kbd>Enter</kbd>               | Select

##### Game controller

  Button/Stick                 |   Action
-------------------------------|-----------
Left and right stick, DPad     | Move
<kbd>A</kbd>                   | Select/Shoot
Trigger and shoulder buttons   | Shoot
<kbd>Back</kbd>                | Pause game

### Settings file

Settings file name is `space_boss_battles_settings.txt` and it will be created or
overwritten to the current working directory every time the game exits.

You shouldn't have to change the file manually, unless
you want to change game controller mapping for a game controller which SDL2 library doesn't provide a
default game controller mapping. Only those game controllers which doesn't
have a default game controller mapping will appear to the file with default mapping defined in game's source code.

When modifying game controller mapping, you might want to start the game with option to print
joystick events to the command line to identify correct button or axis numbers.

### Music

Game supports playing music, but currently there aren't any music included with the game.
By default game tries to play `music.ogg` named file in current working directory. You can
override this with file path specified with command line option.

Supported music file formats are the ones which SDL2_mixer library supports.

### Command line options

  Option                       |   Description
-------------------------------|--------------------------------------------
--help or -h                   | Prints help text about command line options.
--fps                          | Print fps number to command line.
--joystick-events              | Print joystick events to command line.
--music FILE_PATH              | Set path to music file which game tries to play.

If running the game with Cargo, you can set command line options like this:
```
cargo run --release -- --joystick-events --music /path/to/music/file
```

### Troubleshooting

If the game crashes, doesn't start or there is no sound effects, start the game from the command line and check the error messages.

Typical reasons for game to crash:

* Window or OpenGL initialization fails.
* All textures are not found.
* All sound effects are not found.

## Building and running

### Linux

1. Install Rust, https://www.rust-lang.org/en-US/index.html

2. Install SDL2 and SDL2_mixer libraries and developer packages from package repositories. On Ubuntu 16.04 that happens with this command.

```
sudo apt-get install libsdl2-2.0-0 libsdl2-dev libsdl2-mixer-2.0-0 libsdl2-mixer-dev
```

3. Clone (or download) the repository

```
git clone https://github.com/jutuon/space-boss-battles.git
```

4. Change working directory to repository directory

```
cd space-boss-battles
```

5. Build and run the game with Cargo

```
cargo run --release
```

To run the game with OpenGL ES 2.0, build and run with this command
```
cargo run --release --features "gles"
```

### Raspberry Pi

Note that you may need to set GPU RAM size to at least 128 MB to run the game with frame rate of 60 fps at 1080p resolution.

#### Raspberry Pi 2 and 3

1. Enable experimental OpenGL driver from `raspi-config`.

2. Follow Linux instructions.

3. Build the game with OpenGL ES support.

#### Raspberry Pi 1 and Zero

See this page for more information: [SDL2 Library and Raspberry Pi - Raspberry Pi 1 and Zero](https://github.com/jutuon/raspberry-pi-game-development/tree/master/sdl2#raspberry-pi-1-and-zero)

Note that you should only use game controllers with SDL2 rpi video driver, see the link above for more details.

If you use the build script from link above, run the script like this:

```
./build_sdl2.py --build-sdl2 --build-mixer --add-profile-variables
```

Before building and running the game with Cargo, set `LIBRARY_PATH` and `LD_LIBRARY_PATH` environment variables, and
then build and run the game with this command:

```
SDL_VIDEODRIVER=rpi cargo run --release --features "gles"
```

## Documentation

You can generate documentation and open it by running this command in
root directory of the repository.
```
cargo doc --open
```
Note that `cargo doc` will only create documentation for public items in the code.

See also the documentation in repository's documentation directory.

## License

This project's code is licensed under

* Apache License, Version 2.0, [LICENSE-APACHE](https://github.com/jutuon/space-boss-battles/blob/master/LICENSE-APACHE)

or

* MIT License, [LICENSE-MIT](https://github.com/jutuon/space-boss-battles/blob/master/LICENSE-MIT)

You can choose either one from licenses above when you do something with this code.

Image and audio files are licensed under [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).

## Contributions

Contributions will be licensed as stated in License section of this file.
