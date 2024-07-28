



#version 140

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;
uniform float u_time;
uniform float u_input;
uniform vec2 u_resolution;


void main() {

    vec4 c = texture(tex, v_tex_coords);
    float u = (c.r+c.g+c.b)/3.0;

    if (u_input > u) {
        color = vec4(0.0);
    } else {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
}