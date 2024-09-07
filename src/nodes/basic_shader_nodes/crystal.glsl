

#version 420 core

in vec2 v_tex_coords;
out vec4 color;

highp float rand(vec2 co)
{
    highp float a = 12.9898;
    highp float b = 78.233;
    highp float c = 43758.5453;
    highp float dt= dot(co.xy ,vec2(a,b));
    highp float sn= mod(dt,3.14);
    return fract(sin(sn) * c);
}

vec2 rand_pos(vec2 p, float size) {
    if (rand(p * 100.0 * 2.321341) > 0.7) {
        return vec2(0.0,0.0);
    }  
    return round(vec2(p.x + mod(17*size * rand(p), size*2.0), p.y + mod(12*size * rand(p * 3.141592), size * 2.0)));
}

uniform sampler2D tex;          // texture to blur

            uniform float u_time;
            uniform float u_input;
            uniform vec2 u_resolution;

          // blur radius
//---------------------------------------------------------------------------
void main()
    {

        color = vec4(0.4353, 1.0, 0.0, 1.0);

        float size = ((u_resolution.x + u_resolution.y)/2.0) / round(sqrt(u_input));

        vec2 pos =rand_pos(u_resolution * v_tex_coords - mod(u_resolution * v_tex_coords, size), size);
        vec2 og = u_resolution * v_tex_coords;
        for(int x=-3;x<=3;++x) {
            for(int y=-3;y<=3;y++) {
                vec2 new_pos = rand_pos(round(u_resolution * v_tex_coords - mod(u_resolution * v_tex_coords, size) +  vec2(float(x),float(y)) * size), size);
                if (distance(og, pos) > distance(og, new_pos)) {
                    pos = new_pos;
                }
            }
        }

        color = texture2D(tex, pos/u_resolution);

        

    }