#version 330 core

/*
src/shaders/gl/color-fragment.glsl, 2017-08-16

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

// OpenGL 3.3 fragment shader for rendering with specific color.

out vec4 color_out;

uniform vec3 color;

void main() {
    color_out = vec4(color,1.0);
}