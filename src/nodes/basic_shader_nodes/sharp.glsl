

#version 420 core

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;// texture to blur

uniform float u_time;
uniform float u_input;
uniform vec2 u_resolution;


vec4 sharpen(sampler2D tex, vec2 coords, vec2 renderSize, float sharpnes) {
  float dx = 1.0 / renderSize.x;
  float dy = 1.0 / renderSize.y;
  vec4 sum = vec4(0.0);
  sum += ( sharpnes * -1.) * texture2D(tex, coords + vec2( -1.0 * dx , 0.0 * dy));
  sum += ( sharpnes * -1.) * texture2D(tex, coords + vec2( 0.0 * dx , -1.0 * dy));
  sum += ( sharpnes * 5. )* texture2D(tex, coords + vec2( 0.0 * dx , 0.0 * dy));
  sum += ( sharpnes * -1.) * texture2D(tex, coords + vec2( 0.0 * dx , 1.0 * dy));
  sum += ( sharpnes * -1.) * texture2D(tex, coords + vec2( 1.0 * dx , 0.0 * dy));
  return sum;
}


// blur radius
//---------------------------------------------------------------------------
void main()
{



    color = sharpen(tex, v_tex_coords, u_resolution, u_input);

        }