#version 100

/*
src/shaders/gles/vertex-shader-tilemap-gles.glsl, 2017-08-01

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
uniform vec3 tile_info;

void main() {
    gl_Position = P * M * vec4(vertex, 1.0);

    float s = texture_coordinates_attribute.s*tile_info.z + tile_info.x;
    float t = texture_coordinates_attribute.t*tile_info.z + tile_info.y;

    texture_coordinates = vec2(s,-t);
}