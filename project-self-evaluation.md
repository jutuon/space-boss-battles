# Project self-evaluation

Date: 2017-08-28

## Abstract

I rewrote my old 2D arcade space shooter game Space Boss Battles in Rust programming language. Programming went quite well and I didn't
have any major difficulties even if I didn't have much programming experience with Rust.


## Table of contents

1. [Project description, background and goals]()
2. [Implementation]()
3. [Self-evaluation]()
4. [List of sources used during the project]()
5. [Information for project maintainers]()
6. [Future of the project]()
7. [Conclusions]()


## 1. Project description, background and goals

Project's aim was to do something enough challenging to learn the Rust programming language and create a game that will
run on a low-end hardware like Raspberry Pi Model B (512MB RAM). I decided to recreate my old game, Space Boss Battles, that
was written in the CoolBasic programming language. Making an rewrite of an existing game also had that upside I wouldn't
have to create a new textures and sound effects.

The roots of this project kinda start about an year ago when I recreated this same game with Unity3D.

### The Unity3D version of Space Boss Battles

My old game needed a proper rewrite: frame rate was locked to 40 fps, no full screen support, only software rendering and Windows was the only supported
platform. Back then when I did rewrite the game with Unity3D, I was also interested in mobile game development.
Unity3D seemed like an interesting choice for doing the rewrite, easy
exporting to multiple platforms including mobile, GPU rendering and C# support.
It also had tools to make 3D games more easily.

After a couple of weeks I had working build with kinda the same
feature set like the original game. It ran well on my desktop computer, but on my Android smartphone which I had back then, the
Huawei U8800 Ideos X5, it ran poorly. The fact that the game and/or Unity3D ran poorly
on my smartphone was not nice and the game wasn't designed to play with a touch screen, so I kinda lost motivation to the project.

### Raspberry Pi games with native code

Some months later I got an idea that I should make games for Raspberry Pi. At that time I only had the Model B (512 MB RAM) version of
Raspberry Pi. I thought that Raspberry Pi's ARMv6 CPU would be too slow for running a game on top of any kind of virtual machines with
garbage collection, so I started to look out for best languages and libraries for making games which will be compiled to native code.

SDL2 and its built in Renderer had Raspberry Pi support, so started experimenting with the C++ and SDL2. I made a simple benchmarking program
what rendered moving game objects that would bounce from sides of the screen. It worked ok on Raspberry Pi, about 150 objects was the limit when
frame rate went under 60 fps and that was on 1080p resolution. At the time being I didn't know how to use OpenGL ES, so I couldn't test if I could get more
performance using directly OpenGL ES, rather than an SDL2 renderer API which was rendering with OpenGL ES.

After that I started to think should I change the C++ to something more nicer and modern and something that will be more useful in the future.
I could learn the modern C++ or learn Rust. Rust seemed more promising and easier to learn, and I wouldn't have to mess with header files.

I created that same benchmarking program with Rust and SDL2 bindings for it. It went quite well. After that I did try to start recreating Space Boss Battles
with SDL2's built in 2D renderer. I ran to some design issues and there was no time to continue that project.

### Retry with OpenGL (and OpenGL ES)

After two computer graphics courses this spring, I now did have the knowledge to utilize the power of the GPU with OpenGL API, so I started
planing to retrieve my game programming project. I could make my own renderer for the game and I could get some credits from coding the game
thanks to the programming project course at my university (that is why I'm writing this self-evaluation document by the way).

First I red the new Rust book, to recall how Rust code was written and to learn Rust programming concepts better than the last time.
After that I did some experiments with using OpenGL from Rust code and then started writing the game. More about the game's current structure in
the following chapter.


## 2. Implementation

The following is an overview of game's current design.

### Component diagram of main components

Color | Meaning
------|------------------
White | My component
Gray  | External component
Blue  | File

Note that at game initialization, more components depend on SDL2. Also SDL2 Rust binding's types like `Button` and `Keycode` are
used in the components so those make additional dependency and some components are borrowed to each other at updates, like `AudioManager` is borrowed to
`Logic` when updating the game logic.

### Component interfaces

Traits `Renderer`, `SoundEffectPlayer` and `Input`, make `OpenGLRenderer`, `AudioManager` and `InputManager` easily replaceable.


### Crates currently used

* cgmath
* rand
* image
* sdl2
* gl_generator


## 3. Self-evaluation

The project was quite successful. I learned basics of the Rust and the game works well on Raspberry Pi.

I didn't have any major code design issues with Rust, what was opposite what I thought before starting the project.
But I guess there are still improvements to be made to the code. It takes time to learn how to fully utilize Rust's type system.

I learned that there should be two separate projection matrixes for GUI and game logic and
methods to get display size in world coordinates for both GUI and game logic. That would allow easy GUI and game scaling.
Currently there is only one projection matrix and method for getting display width in world coordinates.

Because of learning purposes, I decided to make my own GUI toolkit and OpenGL wrapper library. This also allowed me to
build any features that the game needed.

My original plans had deadline for the project at end of August. I managed to finnish the game before that.

And yes, its hard to design software that has lot features in a way which keeps maintainability at acceptable rate, just like
how its told at lectures.

## 4. List of sources used during the project

* Crate documentation
* Rust standard library documentation
* https://learnopengl.com/

## 5. Information for project maintainers

See the project's readme file for building instructions.

## 6. Future of the project

There are still things to do listed at project readme file. Also changing my GUI toolkit and OpenGL wrapper to proper libraries
like `conrod` and `glium`, should be considered.

## 7. Conclusions

Project went quite well, but like in every software project, there are still improvements to be made.