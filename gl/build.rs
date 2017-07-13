/*
gl/build.rs, 2017-07-13

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/


//! Build script for generating OpenGL and OpenGL ES
//! bindings for gl crate.


extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks, GlobalGenerator};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Write, Read};


fn main() {
    let current_dir_path = env::current_dir().unwrap().into_os_string();

    create_bindings(OpenGLApi::Gl, &mut Path::new(&current_dir_path).join("src/gl_generated.rs"));
    create_bindings(OpenGLApi::Gles, &mut Path::new(&current_dir_path).join("src/gl_es_generated.rs"));
}

/// OpenGL API type
#[derive(Debug)]
enum OpenGLApi {
    Gl,
    Gles,
}

/// Writes bindings for OpenGL 3.3 or OpenGL ES 2.0
/// to a specific file.
///
/// If file already exists, this function will not overwrite it.
/// This function will check if file already exists with function
/// `file_exists_and_contains_message`
///
fn create_bindings(opengl: OpenGLApi, file_path: &mut PathBuf) {

    let message: &'static [u8] = b"//
//
// This file is auto generated.
//
//
";

    if file_exists_and_contains_message(&file_path, message) {
        return;
    }

    let mut file = File::create(&file_path).unwrap();

    file.write(message).unwrap();

    let registry;

    match opengl {
        OpenGLApi::Gl => {
             registry = Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, []);
        },
        OpenGLApi::Gles => {
             registry = Registry::new(Api::Gles2, (2, 0), Profile::Core, Fallbacks::All, []);
        },
    }

    registry.write_bindings(GlobalGenerator, &mut file).unwrap();

}

/// Returns true if file exists and starts with specific bytes.
///
/// This function will panic if bytes in the beginning of the file
/// does not match.
fn file_exists_and_contains_message(file_path: &PathBuf, message: &[u8]) -> bool {
        let _file_result = File::open(&file_path);

        if let Ok(mut file) = _file_result {
            let mut vector = Vec::with_capacity(message.len());

            for _ in message {
                vector.push(0);
            }

            let mut file_content: Box<[u8]> = vector.into_boxed_slice();

            file.read(&mut file_content).unwrap();

            if file_content.iter().ne(message) {
                panic!("Unknown or modified generated bindings file found, remove {:?} and build again", file_path);
            } else {
                return true;
            }
        }

        return false;
}