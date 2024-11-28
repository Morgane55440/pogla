#version 450
layout( quads,equal_spacing,ccw) in;


out vec3 pos;
out float transparency;
out vec2 tex_coord;

uniform float anim_time;
uniform float water_size;

uniform float daytime;

uniform float aspect_ratio;

uniform float seed;

uniform mat4 model_view_matrix;
mat4 projection_matrix = mat4(
			      250.00000, 0.00000, 0.00000, 0.00000,
			      0.00000, 250.00000, 0.00000, 0.00000,
			      0.00000, 0.00000, -1.02020, -10.10101,
			      0.00000, 0.00000, -1.00000, 0.00000);

void main(){
  vec4 p1 = mix(gl_in[0].gl_Position,gl_in[1].gl_Position,gl_TessCoord.x);
  vec4 p2 = mix(gl_in[3].gl_Position,gl_in[2].gl_Position,gl_TessCoord.x);
  vec4 p = mix(p1 ,p2 ,gl_TessCoord.y);
  


    float near = 2.4;
    float far = 3.6;
  if (length(p) < near) {
    transparency = 1.0;
  } else if (length(p) > far) {
    transparency = 0.0;
  } else {
    transparency = 1.0 - ((length(p) - near) / (far - near)) * ((length(p) - near) / (far - near));
  }
  
  pos = p.xyz;

  float tex_x = mod(p.x * water_size, 1.0) / 40;
  float tex_y = mod(p.z * water_size, 1.0);

  float xoffset = floor(mod(floor(anim_time * 20.0), 40.0)) / 40.0;

  tex_coord = vec2(tex_x + xoffset, tex_y);


  gl_Position = projection_matrix * model_view_matrix * p;
  if (aspect_ratio < 1.0) {
    gl_Position.y *= aspect_ratio;

  } else {
    gl_Position.x /= aspect_ratio;
  }
}