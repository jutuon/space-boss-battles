/*
gl/src/lib.rs, 2017-07-19

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

//! Light wrapper library around raw OpenGL bindings from `gl_generator` crate.
//!
//! # Features
//! * Supports OpenGL 3.3 and OpenGL ES 2.0.
//! * Wrappers around some OpenGL objects like shaders.
//! * More safer interface to OpenGL.
//!
//! # Notes
//! This library is primarily created because of learning purposes and
//! to make writing OpenGL renderer in Rust nicer because otherwise there would be
//! a lot of unsafe blocks.
//!
//! This is only a light wrapper, so for additional safety use `glium` crate instead.
//!
//! To enable OpenGL ES 2.0 support, compile this library with `gles` feature enabled.
//!
//! # Examples
//! See code of OpenGL renderer in Space Boss Battles for examples.


extern crate cgmath;

mod gl_es_generated;
mod gl_generated;

mod gl_wrapper;

pub use gl_wrapper::*;