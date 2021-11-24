#version 440 core

layout(location = 0) in vec3 charge_color;
layout(location = 1) in float time_to_live;
layout(location = 2) in vec2 frag_pos;

layout(location = 0) out vec4 out_color;

void main() {
    float dist = abs(length(frag_pos) - 1);
    float power = 1.0/exp(dist*3.0);
    out_color = vec4(charge_color, power*time_to_live*time_to_live);
}
