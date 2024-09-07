

#version 420 core

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;

uniform vec3 weights;

uniform int f_type;



void main()
{

    vec4 c = texture2D(tex, v_tex_coords);
    vec3 c2 = c.rgb * weights;

    float b = 0.0;
    if (f_type == 0) {
        b = (max(c2.r, max(c2.g, c2.b)) + max(c2.r, max(c2.g, c2.b)))/2.0;
    }else if (f_type == 1) {
        b = (c2.r + c2.g + c2.b)/3.0;
    }

    color = vec4(vec3(b), c.a);

}