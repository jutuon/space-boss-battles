#version 330 core

/*
src/shaders/gl/fragment-shader.glsl, 2017-07-23

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

in vec2 texture_coordinates;
out vec4 color;

uniform sampler2D texture_sampler;

void main() {
    color = texture(texture_sampler, texture_coordinates);

    if (color.a < 0.5) {
        discard;
    }
}