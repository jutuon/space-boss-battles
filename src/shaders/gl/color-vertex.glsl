#version 330 core

/*
src/shaders/gl/color-vertex.glsl, 2017-08-06

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

in vec3 vertex;

uniform mat4 M;
uniform mat4 P;

void main() {
    gl_Position = P * M * vec4(vertex, 1.0);
}
