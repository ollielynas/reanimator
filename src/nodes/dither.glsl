


in vec2 v_tex_coords;
out vec4 color;

uniform int size;
uniform vec2 u_resolution;


vec3 rgb2hsv(vec3 c)
{
    vec4 K=vec4(0.,-1./3.,2./3.,-1.);
    vec4 p=mix(vec4(c.bg,K.wz),vec4(c.gb,K.xy),step(c.b,c.g));
    vec4 q=mix(vec4(p.xyw,c.r),vec4(c.r,p.yzx),step(p.x,c.r));
    
    float d=q.x-min(q.w,q.y);
    float e=1.e-10;
    return vec3(abs(q.z+(q.w-q.y)/(6.*d+e)),d/(q.x+e),q.x);
}
float luma(vec4 c){
    return rgb2hsv(vec3(c.r,c.g,c.b)).b;
}
float luma(vec3 c){
    return rgb2hsv(vec3(c.r,c.g,c.b)).b;
}





float dither8x8(vec2 position,float brightness){
    int x=int(mod(position.x,8.));
    int y=int(mod(position.y,8.));
    int index=x+y*8;
    float limit=0.;
    
    if(x<8){
        if(index==0)limit=.015625;
        if(index==1)limit=.515625;
        if(index==2)limit=.140625;
        if(index==3)limit=.640625;
        if(index==4)limit=.046875;
        if(index==5)limit=.546875;
        if(index==6)limit=.171875;
        if(index==7)limit=.671875;
        if(index==8)limit=.765625;
        if(index==9)limit=.265625;
        if(index==10)limit=.890625;
        if(index==11)limit=.390625;
        if(index==12)limit=.796875;
        if(index==13)limit=.296875;
        if(index==14)limit=.921875;
        if(index==15)limit=.421875;
        if(index==16)limit=.203125;
        if(index==17)limit=.703125;
        if(index==18)limit=.078125;
        if(index==19)limit=.578125;
        if(index==20)limit=.234375;
        if(index==21)limit=.734375;
        if(index==22)limit=.109375;
        if(index==23)limit=.609375;
        if(index==24)limit=.953125;
        if(index==25)limit=.453125;
        if(index==26)limit=.828125;
        if(index==27)limit=.328125;
        if(index==28)limit=.984375;
        if(index==29)limit=.484375;
        if(index==30)limit=.859375;
        if(index==31)limit=.359375;
        if(index==32)limit=.0625;
        if(index==33)limit=.5625;
        if(index==34)limit=.1875;
        if(index==35)limit=.6875;
        if(index==36)limit=.03125;
        if(index==37)limit=.53125;
        if(index==38)limit=.15625;
        if(index==39)limit=.65625;
        if(index==40)limit=.8125;
        if(index==41)limit=.3125;
        if(index==42)limit=.9375;
        if(index==43)limit=.4375;
        if(index==44)limit=.78125;
        if(index==45)limit=.28125;
        if(index==46)limit=.90625;
        if(index==47)limit=.40625;
        if(index==48)limit=.25;
        if(index==49)limit=.75;
        if(index==50)limit=.125;
        if(index==51)limit=.625;
        if(index==52)limit=.21875;
        if(index==53)limit=.71875;
        if(index==54)limit=.09375;
        if(index==55)limit=.59375;
        if(index==56)limit=1.;
        if(index==57)limit=.5;
        if(index==58)limit=.875;
        if(index==59)limit=.375;
        if(index==60)limit=.96875;
        if(index==61)limit=.46875;
        if(index==62)limit=.84375;
        if(index==63)limit=.34375;
    }
    
    return brightness<limit?0.:1.;
}



float dither4x4(vec2 position,float brightness){
    int x=int(mod(position.x,4.));
    int y=int(mod(position.y,4.));
    int index=x+y*4;
    float limit=0.;
    
    if(x<8){
        if(index==0)limit=.0625;
        if(index==1)limit=.5625;
        if(index==2)limit=.1875;
        if(index==3)limit=.6875;
        if(index==4)limit=.8125;
        if(index==5)limit=.3125;
        if(index==6)limit=.9375;
        if(index==7)limit=.4375;
        if(index==8)limit=.25;
        if(index==9)limit=.75;
        if(index==10)limit=.125;
        if(index==11)limit=.625;
        if(index==12)limit=1.;
        if(index==13)limit=.5;
        if(index==14)limit=.875;
        if(index==15)limit=.375;
    }
    
    return brightness<limit?0.:1.;
}



float dither2x2(vec2 position,float brightness){
    int x=int(mod(position.x,2.));
    int y=int(mod(position.y,2.));
    int index=x+y*2;
    float limit=0.;
    
    if(x<8){
        if(index==0)limit=.25;
        if(index==1)limit=.75;
        if(index==2)limit=1.;
        if(index==3)limit=.50;
    }
    
    return brightness<limit?0.:1.;
}





uniform sampler2D tex;
void main(){
    vec4 c =texture(tex,v_tex_coords);

    float brightness = rgb2hsv(vec3(c.r,c.g,c.b)).b;

    if (size == 2) {
        color = vec4(vec3(dither2x2(v_tex_coords * u_resolution, brightness)), c.a);
    }
    if (size == 4) {
        color = vec4(vec3(dither4x4(v_tex_coords * u_resolution, brightness)), c.a);
    }
    if (size == 8) {
        color = vec4(vec3(dither8x8(v_tex_coords * u_resolution, brightness)), c.a);
    }

}