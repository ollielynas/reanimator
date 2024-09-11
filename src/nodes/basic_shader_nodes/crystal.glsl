

#version 140 core

in vec2 v_tex_coords;
out vec4 color;

highp float rand(vec2 co)
{
    highp float a=12.9898;
    highp float b=78.233;
    highp float c=43758.5453;
    highp float dt=dot(co.xy,vec2(a,b));
    highp float sn=mod(dt,3.14);
    return fract(sin(sn)*c);
}

vec2 rand_pos(ivec2 p,float size){
    if(rand(vec2(p)*432.32)>.8){
        return vec2(0.,0.);
    }
    return vec2(
            p.x*size
                +mod(17*size*rand(vec2(p)),size*2.),
            p.y*size
                +mod(12*size*rand(p*3.141592),size*2.)        
        );
}

uniform sampler2D tex;// texture to blur

uniform float u_time;
uniform float u_input;
uniform vec2 u_resolution;


void main()
{
    float size=round(
        (u_resolution.x+u_resolution.y)/2.)/(sqrt(u_input));

    ivec2 og_pos = ivec2(floor(u_resolution*v_tex_coords/size));

    vec2 pos=rand_pos(og_pos, size);
    vec2 og=u_resolution*v_tex_coords;
    
    for(int x=-3;x<=3;++x){
        for(int y=-3;y<=3;y++){

            vec2 new_pos=rand_pos(og_pos + ivec2(x,y), size);

            if (clamp(new_pos, vec2(0.0), u_resolution) == new_pos && distance(og,pos)>distance(og,new_pos)){
                pos=new_pos;
            }
        }
    }


    color=texture2DLod(tex,pos/u_resolution, 0.0);
    
}