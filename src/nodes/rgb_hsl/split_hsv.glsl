#version 420

in vec2 v_tex_coords;
out vec4 color;

uniform bool raw;

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
uniform int hsva_index;

void main(){
    
    float channel=0.;
    vec4 px=texture(tex,v_tex_coords);
    
    vec3 hsv=rgb2hsv(vec3(px.r,px.g,px.b));
    
    
    
    if (raw) {
        if(hsva_index==0){
        color = vec4(vec3(hsv.r), px.a);
    }else if(hsva_index==1){
        color = vec4(vec3(hsv.g), px.a);
    }else if(hsva_index==2){
        color = vec4(vec3(hsv.b), px.a);
    }
    } else {
        if(hsva_index==0){
        // channel = px.r;
        hsv.g=1.;
        hsv.b=1.;
    }else if(hsva_index==1){
        // hsv.r = 0.0;
        hsv.b=1.;
    }else if(hsva_index==2){
        hsv.r=0.;
        hsv.g=0.;
    }
        color=vec4(hsv2rgb(hsv),px.a);
    }

    if(hsva_index==3){
        color=vec4(px.a);
    }
    
}