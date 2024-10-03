

#version 420 core

const float SAMPLES = 30.0;

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;          // texture to blur

            uniform float u_time;
            uniform float u_input;
            uniform vec2 u_resolution;

          // blur radius
//---------------------------------------------------------------------------
void main()
    {
    float r = u_input;
    vec2 pos = v_tex_coords;
    float xs = (u_resolution.x);
    float ys = (u_resolution.y);
    float x,y,xx,yy,rr=r*r,dx,dy,w,w0;
    w0=0.3780/pow(r,1.975);
    float weight_total = 0.0;
    vec2 p;
    vec4 col=vec4(0.0, 0.0, 0.0, 0.0);
    

    for (float x2=-SAMPLES;x2<SAMPLES;x2++) {
        dx=1.0/xs;
        p.x=(pos.x)+(x*dx);
        x=(x2/SAMPLES) * r;
        xx=x*x;
        for (float y2=-SAMPLES;y2<SAMPLES;y2++) {
        dy=1.0/ys;
        p.y=(pos.y)+(y*dy);
        y=(y2/SAMPLES) * r;
        yy=y*y;
        if (xx+yy<=rr)
        {
            
            // w=w0*(1.0+((-xx-yy)/(2.0*rr))+(((-xx-yy)/(2.0*rr))*((-xx-yy)/(2.0*rr)))/2.0);
            w=w0*exp((-xx-yy)/(2.0*rr));
            weight_total += w;
            col+=texture2D(tex,p)*w;
        }
    }
    }
    color=col/weight_total;

    // for (dx=1.0/xs,x=-r,p.x=(pos.x)+(x*dx);x<=r;x+=pow(sample_disperse, 0.5),p.x+=dx){ 
    //     xx=x*x;
    //  for (dy=1.0/ys,y=-r,p.y=(pos.y)+(y*dy);y<=r;y+=pow(sample_disperse, 0.5),p.y+=dy){ 
    //     yy=y*y;
    //   if (xx+yy<=rr)
    //     {
    //     w=w0*exp((-xx-yy)/(2.0*rr));
    //     weight_total += w;
    //     col+=texture2D(tex,p)*w;
    //     }}}
    }