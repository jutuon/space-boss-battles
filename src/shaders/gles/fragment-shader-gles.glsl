#version 100

/*
src/shaders/gles/fragment-shader-gles.glsl, 2017-08-16

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

// OpenGL ES 2.0 fragment shader for rendering a texture.

precision mediump float;

varying vec2 texture_coordinates;

uniform sampler2D texture_sampler;

void main() {
    gl_FragColor = texture2D(texture_sampler, texture_coordinates);

    if (gl_FragColor.a < 0.5) {
        discard;
    }
}