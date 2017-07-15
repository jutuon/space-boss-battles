#version 330 core

/*
src/shaders/color-fragment.glsl, 2017-07-15

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

out vec4 color;

uniform vec3 color_uniform;

void main() {
    color = vec4(color_uniform,1.0);
}