#version 330 core

/*
src/shaders/color-vertex.glsl, 2017-07-15

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

layout (location = 0) in vec3 vertex;

uniform mat4 M;
uniform mat4 P;

void main() {
    gl_Position = P * M * vec4(vertex, 1.0);
}