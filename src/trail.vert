#version 440 core

layout(location = 0) in vec2 vert_position;
layout(location = 1) in vec2 trail_position;
layout(location = 2) in float time_to_live;
layout(location = 3) in float radius;
layout(location = 4) in float charge;

layout(location = 0) out vec3 charge_color;
layout(location = 1) out float out_time_to_live;
layout(location = 2) out vec2 orig_pos;

layout(std140, set = 0, binding = 0) uniform transform {
    mat4 m;
};

vec3 get_charge_color(float charge)
{
    if (charge == 0) {
        return vec3(1.0, 1.0, 1.0);
    } else if (charge < 0) {
        return vec3(0.286, 0.322, 1.0);
    } else {
        return vec3(0.961, 0.0, 0.302);
    }
}

void main() {
    orig_pos = vert_position;
    gl_Position = m * vec4(vert_position * (radius / 2) + trail_position, 0, 1);
    charge_color = get_charge_color(charge);
    charge_color = charge_color * charge_color;
    out_time_to_live = time_to_live;
}
