# Space Boss Battles

Space Boss Battles is 2D arcade space shooter game. This version of the game is complete rewrite in Rust programming language. Originally game was written in CoolBasic programming language.

This project is also my first proper project written in Rust and will also serve as coursework for programming project course.

## Features

General features:

- [x] OpenGL 3.3 and OpenGL ES 2.0 support
- [ ] Support for display aspect ratios of 4:3 and 16:9
- [ ] Basic menu system for main menu and settings
- [ ] 4 levels


Platforms:

- [x] Linux
- [ ] Windows
- [ ] Mac
- [ ] Android
- [ ] Web



## Building and running

1. Install dependencies
  - Rust, https://www.rust-lang.org/en-US/index.html
  - SDL2 library, https://www.libsdl.org/

2. Clone (or download) the repository

```
git clone https://github.com/jutuon/space-boss-battles.git
```

3. Change working directory to repository directory

```
cd space-boss-battles
```

4. Build and run the game with Cargo

```
cargo run --release
```

To run the game with OpenGL ES 2.0, build and run with this command
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

You can choose which licence you use when you do something with this code.

Image and audio files are licensed under [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).

## Contributions

Contributions will be licensed as stated in Licence section of this file.
