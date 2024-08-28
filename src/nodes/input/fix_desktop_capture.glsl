


in vec2 v_tex_coords;
out vec4 color;


uniform sampler2D tex;
void main(){
    // memoryBarrier();
    vec2 cord = vec2(v_tex_coords.x, 1.0-v_tex_coords.y);

    vec4 c =texture(tex,cord);

    color = vec4(c.r,c.g,c.b, c.a);

}