/*
gl/src/lib.rs, 2017-07-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

extern crate cgmath;

mod gl_es_generated;
mod gl_generated;

mod gl_wrapper;

pub use gl_wrapper::*;