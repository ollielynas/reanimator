

#version 420 core

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;          // texture to blur

uniform float u_time;
uniform float u_input;
uniform vec2 u_resolution;

const vec2 points[109] = vec2[](
vec2(0.0,0.0),
vec2(0.0,-1.0),
vec2(0.0,1.0),
vec2(-1.0,-1.0),
vec2(-1.0,1.0),
vec2(-1.0,0.0),
vec2(1.0,-1.0),
vec2(1.0,1.0),
vec2(1.0,0.0),
vec2(-1.0,2.0),
vec2(-1.0,-2.0),
vec2(1.0,2.0),
vec2(1.0,-2.0),
vec2(2.0,2.0),
vec2(2.0,1.0),
vec2(2.0,-2.0),
vec2(2.0,-1.0),
vec2(2.0,0.0),
vec2(-2.0,2.0),
vec2(-2.0,1.0),
vec2(-2.0,-2.0),
vec2(-2.0,-1.0),
vec2(-2.0,0.0),
vec2(0.0,2.0),
vec2(0.0,-2.0),
vec2(3.0,1.0),
vec2(3.0,2.0),
vec2(3.0,-2.0),
vec2(3.0,-1.0),
vec2(3.0,0.0),
vec2(2.0,-3.0),
vec2(2.0,3.0),
vec2(1.0,-3.0),
vec2(1.0,3.0),
vec2(0.0,-3.0),
vec2(0.0,3.0),
vec2(-1.0,-3.0),
vec2(-1.0,3.0),
vec2(-3.0,1.0),
vec2(-3.0,2.0),
vec2(-3.0,-2.0),
vec2(-3.0,-1.0),
vec2(-3.0,0.0),
vec2(-2.0,-3.0),
vec2(-2.0,3.0),
vec2(1.0,-4.0),
vec2(1.0,4.0),
vec2(0.0,-4.0),
vec2(0.0,4.0),
vec2(3.0,-3.0),
vec2(3.0,3.0),
vec2(-1.0,-4.0),
vec2(-1.0,4.0),
vec2(4.0,0.0),
vec2(4.0,-2.0),
vec2(4.0,-1.0),
vec2(4.0,1.0),
vec2(4.0,2.0),
vec2(2.0,-4.0),
vec2(2.0,4.0),
vec2(-4.0,0.0),
vec2(-4.0,-2.0),
vec2(-4.0,-1.0),
vec2(-4.0,1.0),
vec2(-4.0,2.0),
vec2(-2.0,-4.0),
vec2(-2.0,4.0),
vec2(-3.0,-3.0),
vec2(-3.0,3.0),
vec2(-1.0,5.0),
vec2(-1.0,-5.0),
vec2(-3.0,5.0),
vec2(-3.0,-4.0),
vec2(-3.0,-5.0),
vec2(-3.0,4.0),
vec2(3.0,5.0),
vec2(3.0,-4.0),
vec2(3.0,-5.0),
vec2(3.0,4.0),
vec2(-5.0,-1.0),
vec2(-5.0,-3.0),
vec2(-5.0,-2.0),
vec2(-5.0,2.0),
vec2(-5.0,3.0),
vec2(-5.0,0.0),
vec2(-5.0,1.0),
vec2(-2.0,5.0),
vec2(-2.0,-5.0),
vec2(5.0,-1.0),
vec2(5.0,-3.0),
vec2(5.0,-2.0),
vec2(5.0,2.0),
vec2(5.0,3.0),
vec2(5.0,0.0),
vec2(5.0,1.0),
vec2(4.0,-4.0),
vec2(4.0,-3.0),
vec2(4.0,3.0),
vec2(4.0,4.0),
vec2(-4.0,-4.0),
vec2(-4.0,-3.0),
vec2(-4.0,3.0),
vec2(-4.0,4.0),
vec2(0.0,5.0),
vec2(0.0,-5.0),
vec2(1.0,5.0),
vec2(1.0,-5.0),
vec2(2.0,5.0),
vec2(2.0,-5.0)
);

          // blur radius
//---------------------------------------------------------------------------
void main()
    {
    float r = u_input;
    vec2 pos = v_tex_coords;

    float x,y,xx,yy,rr=r*r,dx,dy,w,w0;
    w0=0.3780/pow(r,1.975);
    float weight_total = 0.0;
    vec2 p;
    vec4 col=vec4(0.0, 0.0, 0.0, 0.0);

    for (int index=0;index<44;index++) {
        float y = points[index].y;
        float x = points[index].x;
        float xx = x*x;
        yy=y*y;
        p = pos + points[index] / u_resolution;
        float s = (length(texture2D(tex,p) - texture2D(tex,pos))) / 1.732;
        if (s > u_input && index > 2) {
            break;
        }
        w=w0*exp((-xx-yy)/(2.0*rr));
        weight_total += w;
        col+=texture2D(tex,p)*w;
        
    
    }
    color=col/weight_total;

    }