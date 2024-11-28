#version 450

#define PI  3.1415926535897932384626433832795

in vec3 pos;
in float transparency;
in vec2 tex_coord;

uniform float daytime;
uniform sampler2D water_tex;


uniform mat4 model_view_matrix;

layout(location=0) out vec4 output_color;

/* 
tex is in the format :
0 4  8 .. 36
1 5  9 .. 37
2 6 10 .. 38
3 7 11 .. 39

but of course openg tarts bottom left.

thus with t = T % 40,

x = floor(t / 4)

y = 3 - floor(t % 4)


*/

void main() {

 vec3 sun_pos = vec3(20.0, -200 * cos(daytime * PI / 12), 200 * sin(daytime * PI / 12));
  vec3 texcol = max(dot(vec3(0.0,1.0,0.0), normalize(sun_pos - pos)), 0.1) * texture(water_tex, tex_coord).xyz;
  

  output_color = vec4(texcol.r * 0.7, texcol.g * 0.7, texcol.b, transparency);
}