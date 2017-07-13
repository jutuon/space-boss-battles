/*
gl/src/gl_wrapper/mod.rs, 2017-07-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

mod gl_raw {
    pub use gl_generated::*;
}

pub mod shader;
pub mod uniform;
pub mod buffer;
