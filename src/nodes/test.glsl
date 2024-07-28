#version 140

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D base_texture;

uniform sampler2D layer_0;uniform vec2 layer_0_pos;uniform vec2 layer_0_size;uniform vec2 layer_0_target_size

vec4 add_layer(vec2 target_pos,vec2 size,vec2 target_size,vec4 base_color,sampler2D tex_layer){
    if(v_tex_coords.x<target_pos.x||v_tex_coords.y<target_pos.y||v_tex_coords.x>target_pos.x+target_size.x||v_tex_coords.y>target_pos.y+target_size.y){
        return base_color;
    }
    
    vec4 layer_color=texture(tex_layer,((v_tex_coords-target_pos)*size/target_size));
    
    return layer_color;
}

void main(){
    color=texture(base_texture,v_tex_coords);
    color=add_layer(layer_0_pos,layer_0_size,color,layer_0);
}