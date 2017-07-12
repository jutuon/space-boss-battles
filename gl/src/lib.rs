/*
gl/src/lib.rs, 2017-07-12

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0, https://github.com/jutuon/space-boss-battles/LICENCE-APACHE

or

MIT License, https://github.com/jutuon/space-boss-battles/LICENCE-MIT
*/

extern crate cgmath;

mod gl_es_generated;
mod gl_generated;

mod gl_wrapper;

pub use gl_wrapper::*;