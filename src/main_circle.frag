#version 440 core

layout(location = 0) in vec3 charge_color;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = vec4(charge_color, 1);
}
