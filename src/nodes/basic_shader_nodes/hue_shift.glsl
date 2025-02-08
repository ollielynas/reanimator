
#version 420


vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;// texture to blur

uniform float u_time;
uniform float u_input;
uniform vec2 u_resolution;

void main(){

    vec4 c = texture2D(tex, v_tex_coords);

    float deg = u_input;
    float rot = deg/360.0;
    vec3 hsv = rgb2hsv(c.xyz);
    hsv.x = mod(hsv.x + rot, 1.0);

    color = vec4(hsv2rgb(hsv), c.a);
}