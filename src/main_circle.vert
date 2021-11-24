#version 440 core

layout(location = 0) in vec2 vert_position;
layout(location = 1) in vec2 particle_position;
layout(location = 2) in float radius;
layout(location = 3) in float charge;

layout(location = 0) out vec3 charge_color;

void main() {
    gl_Position = vec4(vert_position * (radius / 2) + particle_position, 0, 1);
    charge_color = vec3(1.0, 1.0, 1.0);
}
