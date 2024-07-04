

#version 140

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;
uniform float u_time;
uniform float u_input;
uniform vec2 u_resolution;


void main(){
    color = texture(tex, v_tex_coords);
}