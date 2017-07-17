/*
gl/src/gl_wrapper/mod.rs, 2017-07-17

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

pub mod gl_raw {
    #[cfg(feature = "gles")]
    pub use gl_es_generated::*;

    #[cfg(not(feature = "gles"))]
    pub use gl_generated::*;
}

pub mod shader;
pub mod uniform;
pub mod buffer;
pub mod texture;