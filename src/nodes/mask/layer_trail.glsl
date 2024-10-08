#version 450

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;
uniform sampler2D last_tex;
uniform vec2 u_resolution;
uniform float fade;



void main(){
    
    vec4 last_color=texture(last_tex, v_tex_coords);
    last_color.a = last_color.a * fade; 
    vec4 c = texture(tex, v_tex_coords);

    if (c.a == 0.0) {
        c = last_color;
    }

    color = c;

}