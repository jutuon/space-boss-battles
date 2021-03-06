#version 330 core

/*
src/shaders/gl/vertex-shader.glsl, 2017-08-16

Copyright (c) 2017 Juuso Tuononen

This file is licensed under

Apache License, Version 2.0

or

MIT License
*/

// OpenGL 3.3 vertex shader for rendering a texture.

in vec3 vertex;
in vec2 texture_coordinates_attribute;

out vec2 texture_coordinates;

uniform mat4 M;
uniform mat4 P;

void main() {

    gl_Position = P * M * vec4(vertex, 1.0);

    texture_coordinates = vec2(texture_coordinates_attribute.x,-texture_coordinates_attribute.y);
}
