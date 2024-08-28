#version 420

in vec2 v_tex_coords;
out vec4 color;

vec3 rgb2hsv(vec3 c)
{
    vec4 K=vec4(0.,-1./3.,2./3.,-1.);
    vec4 p=mix(vec4(c.bg,K.wz),vec4(c.gb,K.xy),step(c.b,c.g));
    vec4 q=mix(vec4(p.xyw,c.r),vec4(c.r,p.yzx),step(p.x,c.r));
    
    float d=q.x-min(q.w,q.y);
    float e=1.e-10;
    return vec3(abs(q.z+(q.w-q.y)/(6.*d+e)),d/(q.x+e),q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K=vec4(1.,2./3.,1./3.,3.);
    vec3 p=abs(fract(c.xxx+K.xyz)*6.-K.www);
    return c.z*mix(K.xxx,clamp(p-K.xxx,0.,1.),c.y);
}

uniform sampler2D tex;
uniform vec2 u_resolution;
uniform sampler2D paper;
uniform sampler2D diff;

uniform float r1 = 1.4;
uniform float r2 = 30.0;
uniform float weight = 0.5;
uniform float threshold = 0.1;
uniform float sigma = 10.0;
uniform bool do_threshold = true;
uniform bool greyscale = true;

// blur radius
//---------------------------------------------------------------------------
vec4 diff_g()
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
    
    for(float x2=-25.;x2<25.;x2++){
        dx=1./xs;
        p.x=(pos.x)+(x*dx);
        x=(x2/25.)*r;
        xx=x*x;
        for(float y2=-25.;y2<25.;y2++){
            dy=1./ys;
            p.y=(pos.y)+(y*dy);
            y=(y2/25.)*r;
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
    
    for(float x2=-25.;x2<25.;x2++){
        dx=1./xs;
        p.x=(pos.x)+(x*dx);
        x=(x2/25.)*r;
        xx=x*x;
        for(float y2=-25.;y2<25.;y2++){
            dy=1./ys;
            p.y=(pos.y)+(y*dy);
            y=(y2/25.)*r;
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
    
    float u1=(color1.r+color1.g+color1.b)/3.;
    float u2=(color2.r+color2.g+color2.b)/3.;
    
    if(((1.-weight)*u1-u2*weight)<0.){
        vec4 color5=color1;
        color1=color2;
        color2=color5;
    }
    
    // vec4 color3=((1.0-weight)*color1-color2*weight);
    vec4 color3=((1.-weight)*color1-color2*weight);
    color3.a=1.;
    
    float u=(color3.r+color3.g+color3.b)/3.;
    
    if(greyscale){
        color3=vec4(vec3(u),color3.a);
    }
    
    if(!do_threshold){
        return color3;
    }
    
    if(u<threshold){
        color3=vec4(
            // vec3(
                //     1.0+tanh(sigma * (u-threshold))
            // ),
            vec3(
                1.+tanh(sigma*(color3.r-threshold)),
                1.+tanh(sigma*(color3.g-threshold)),
                1.+tanh(sigma*(color3.b-threshold))
            ),
            1.
        );
    }else{
        color3=vec4(1.,1.,1.,1.);
    }
    
    return color3;
    
}

void main() {
    
    float channel=0.;
    vec4 px=texture(tex,v_tex_coords);
    vec4 px2=texture(paper,v_tex_coords);
    vec4 gauss =1.0 - diff_g();
    
    vec3 hsv=rgb2hsv(vec3(px.r,px.g,px.b));
    vec3 hsv_paper=rgb2hsv(vec3(px2.r,px2.g,px2.b));
    
    vec3 col=hsv2rgb(
        vec3(
            hsv.r,
            clamp((hsv.g), hsv.b*0.1,hsv.b*1.3),
            clamp((hsv.b*0.5+hsv_paper.b * 0.3 + gauss.r*0.2),0.0,1.0)
            ));
    
    color=vec4(col,px.a);

    color.a = px.a;
    
}