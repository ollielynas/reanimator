#version 450

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;
uniform sampler2D last_tex;
uniform vec2 u_resolution;


float color_distance(vec4 c1, vec4 c2) {
    int rmean = (int(c1.r * 255.0) + int(c1.r * 255.0) ) / 2;
    int r = int(c1.r*255.0) - int(c2.r*255.0);
    int g = int(c1.g*255.0) - int(c2.g*255.0);
    int b = int(c1.b*255.0) - int(c2.b*255.0);
    return float(sqrt((((512+rmean)*r*r)>>8) + 4*g*g + (((767-rmean)*b*b)>>8)))/255.0;
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main(){
    
    vec4 last_color=texture(last_tex, v_tex_coords);

    float range = 4.0;
    vec2 best = vec2(0.0);
    float score = 0.0;

    for (float x = -range; x <= range; x += 1.0) {
    for (float y = -range; y <= range; y += 1.0) {
        if (length(vec2(x,y)) <= range) {
            vec4 next_color = texture(tex, v_tex_coords + vec2(x,y)/u_resolution);
            float s = (1.0 - length(last_color - next_color)) / 1.732 * 1.0 - length(vec2(x,y))/length(vec2(range,range));
            if (s > score) {
                score = s;
                best =  vec2(x,y);
            } 
        }
    }
    }

    if (best.x < 0.0) {
        score = 0.0;
    }

    color = vec4(hsv2rgb(vec3((atan(best.x, best.y)+3.14159)/(3.14159*2) , score , length(best)/length(vec2(range)))), last_color.a);

}