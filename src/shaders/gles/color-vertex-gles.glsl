#version 100

/*
src/shaders/gles/color-vertex-gles.glsl, 2017-07-17

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

attribute vec3 vertex;

uniform mat4 M;
uniform mat4 P;

void main() {
    gl_Position = P * M * vec4(vertex, 1.0);
}
