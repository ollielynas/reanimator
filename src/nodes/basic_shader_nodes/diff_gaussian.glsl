

#version 420 core

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;// texture to blur

uniform float u_time;
uniform float r1;
uniform float r2;
uniform float weight;
uniform float threshold;
uniform float sigma;
uniform bool do_threshold;
uniform bool greyscale;
uniform vec2 u_resolution;

// blur radius
//---------------------------------------------------------------------------
void main()
{
    
    float r=r1;


    
    vec2 pos=v_tex_coords;
    float xs=(u_resolution.x);
    float ys=(u_resolution.y);
    float x,y,xx,yy,rr=r*r,dx,dy,w,w0;
    w0=.3780/pow(r,1.975);
    float weight_total=0.;
    vec2 p;
    vec4 col=vec4(0.,0.,0.,0.);
    
    for(float x2=-45.;x2<45.;x2++){
        dx=1./xs;
        p.x=(pos.x)+(x*dx);
        x=(x2/45.)*r;
        xx=x*x;
        for(float y2=-45.;y2<45.;y2++){
            dy=1./ys;
            p.y=(pos.y)+(y*dy);
            y=(y2/45.)*r;
            yy=y*y;
            if(xx+yy<=rr)
            {
                w=w0*exp((-xx-yy)/(2.*rr));
                weight_total+=w;
                col+=texture2D(tex,p)*w;
            }
        }
    }
    
    vec4 color1=col/weight_total;
    
    r=r2;
    
    pos=v_tex_coords;
    xs=(u_resolution.x);
    ys=(u_resolution.y);
    w0=.3780/pow(r,1.975);
    weight_total=0.;
    
    col=vec4(0.,0.,0.,0.);
    
    for(float x2=-45.;x2<45.;x2++){
        dx=1./xs;
        p.x=(pos.x)+(x*dx);
        x=(x2/45.)*r;
        xx=x*x;
        for(float y2=-45.;y2<45.;y2++){
            dy=1./ys;
            p.y=(pos.y)+(y*dy);
            y=(y2/45.)*r;
            yy=y*y;
            if(xx+yy<=rr)
            {
                w=w0*exp((-xx-yy)/(2.*rr));
                weight_total+=w;
                col+=texture2D(tex,p)*w;
            }
        }
    }
    
    vec4 color2=col/weight_total;
    
    // vec4 color3=((1.0-weight)*color1-color2*weight);
    vec4 color3=((1.0-weight)*color1-color2*weight);
    color3.a = 1.0;

    float u = (color3.r+color3.g+color3.b)/3.0;

    if (greyscale) {
        color3 = vec4(vec3(u),color3.a);
    }


    if (!do_threshold) {
        color = color3;
        return;
    }

    
    if (u < threshold) {
        color3 = vec4(
            // vec3(
            //     1.0+tanh(sigma * (u-threshold))
            // ),
            vec3(
                1.0+tanh(sigma * (color3.r-threshold)),
                1.0+tanh(sigma * (color3.g-threshold)),
                1.0+tanh(sigma * (color3.b-threshold))
            ),
            1.0
        );
    }else {
        color3 = vec4(1.0, 1.0, 1.0, 1.0);
    }

    color = color3;

        }