#version 100

/*
src/shaders/gles/fragment-shader-tilemap-gles.glsl, 2017-08-01

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

precision mediump float;

varying vec2 texture_coordinates;

uniform sampler2D texture_sampler;

void main() {
    gl_FragColor = texture2D(texture_sampler, texture_coordinates);

    if (gl_FragColor.a < 0.5) {
        discard;
    }
}