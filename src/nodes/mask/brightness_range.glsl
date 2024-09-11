#version 140

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;
uniform float u_time;
uniform float low;
uniform float high;
// uniform float u_input2;
uniform vec2 u_resolution;


void main() {

    vec4 c = texture(tex, v_tex_coords);
    float u = (c.r+c.g+c.b)/3.0;

    float low2 = low;
    float high2 = high;

    if (low > high) {
        low2 = high;
        high2 = low;
        u = 1.0 - u;
    }

    if (high2 < u || low2 >= u) {
        color = vec4(0.0);
    } else {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
}