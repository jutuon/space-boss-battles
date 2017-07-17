#version 330 core

/*
src/shaders/color-fragment.glsl, 2017-07-17

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

out vec4 color_out;

uniform vec3 color;

void main() {
    color_out = vec4(color,1.0);
}