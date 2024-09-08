
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

uniform sampler2D tex_h;
uniform sampler2D tex_s;
uniform sampler2D tex_v;
uniform sampler2D tex_a;
uniform bool raw;

void main(){

    vec3 h = rgb2hsv(vec3(texture(tex_h, v_tex_coords)));
    vec3 s = rgb2hsv(vec3(texture(tex_s, v_tex_coords)));
    vec3 v = rgb2hsv(vec3(texture(tex_v, v_tex_coords)));

    vec3 c = hsv2rgb(vec3(h.r, s.g, v.b));

    if (raw) {
        c = hsv2rgb(vec3(
            texture(tex_h, v_tex_coords).r,
            texture(tex_s, v_tex_coords).g,
            texture(tex_v, v_tex_coords).b
        ));
    }

    color = vec4(c, texture(tex_a, v_tex_coords).a);

}