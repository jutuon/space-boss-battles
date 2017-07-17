#version 100

/*
src/shaders/gles/vertex-shader-gles.glsl, 2017-07-17

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

attribute vec3 vertex;
attribute vec2 texture_coordinates_attribute;

varying vec2 texture_coordinates;

uniform mat4 M;
uniform mat4 P;

void main() {
    gl_Position = P * M * vec4(vertex, 1.0);

    texture_coordinates = vec2(texture_coordinates_attribute.x,-texture_coordinates_attribute.y);
}
