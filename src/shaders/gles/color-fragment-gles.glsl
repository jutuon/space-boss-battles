#version 100

/*
src/shaders/gles/color-fragment-gles.glsl, 2017-07-17

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

precision mediump float;

uniform vec3 color;

void main() {
    gl_FragColor = vec4(color,1.0);
}