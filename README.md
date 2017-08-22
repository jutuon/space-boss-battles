# Space Boss Battles

Space Boss Battles is 2D arcade space shooter game. This version of the game is complete rewrite in Rust programming language. Originally game was written in CoolBasic programming language.

This project is also my first proper project written in Rust and will also serve as coursework for programming project course.

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
- [ ] Web
- [x] Raspberry Pi

General features:

- [x] Settings file

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

Building the game for Raspberry Pi will work almost like building for Linux but SDL2 library
installation differs from Linux building instructions. See the following link for instructions.

[SDL2 Library and Raspberry Pi](https://github.com/jutuon/raspberry-pi-game-development/tree/master/sdl2)

If you have Raspberry Pi 1 or Zero and you use the building script from link above, set
`LIBRARY_PATH` and `LD_LIBRARY_PATH` environment variables before building and running the
game with Cargo.

Note also that OpenGL 3.3 is not available on Raspberry Pi, so build and run the game with OpenGL ES 2.0 support.
```
cargo run --release --features "gles"
```

## Documentation

You can generate documentation and open it by running this command in
root directory of the repository.
```
cargo doc --open
```


## License

This project's code is licensed under

* Apache License, Version 2.0, [LICENSE-APACHE](https://github.com/jutuon/space-boss-battles/blob/master/LICENSE-APACHE)

or

* MIT License, [LICENSE-MIT](https://github.com/jutuon/space-boss-battles/blob/master/LICENSE-MIT)

You can choose either one from licenses above when you do something with this code.

Image and audio files are licensed under [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).

## Contributions

Contributions will be licensed as stated in License section of this file.
