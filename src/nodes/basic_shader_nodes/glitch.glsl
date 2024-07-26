

#version 420 core

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
    float area = 3*r*r;

    for (float x2=-45.0;x2<45.0;x2++) {
        x=(x2/45.0) * r;
        xx=x*x;
        for (float y2=-45.0;y2<45.0;y2++) {
        y=(y2/45.0) * r;
        yy=y*y;
        if (xx+yy<=rr)
        {
            w=w0*exp((-xx-yy)/(2.0*rr));
            weight_total += w;
            col+=texture2D(tex,p)*w;
        }
    }
    }

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
    color=col/weight_total;
    }