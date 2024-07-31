

#version 420 core

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;// texture to blur

uniform float u_time;
uniform float u_input;
uniform vec2 u_resolution;



// blur radius
//---------------------------------------------------------------------------
void main()
{
    float r=u_input;
    float xs=(u_resolution.x);
    float ys=(u_resolution.y);


    vec2 pos = u_resolution * v_tex_coords;

    vec2 closest_point = vec2(
        pos.x - mod(pos.x, 2 * r * sqrt(2)),
        pos.y - mod(pos.y, 2 * r * sqrt(2))
    ) + r * sqrt(2);

    vec2 pos2 = closest_point / u_resolution;

        vec4 c = texture(tex, pos2);
    float u = (c.r+c.g+c.b)/3.0;


    float d = distance(pos, closest_point);
    float g = 0.0;

    if (d < r * u * u) {
        g = 1.0;
    }

    color = vec4(g);

        }