#version 450
layout( quads,equal_spacing,ccw) in;


out vec4 color;

uniform float anim_time;

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
  

  gl_Position =  p ;

    float near = 2.4;
    float far = 3.6;
  if (length(p) < near) {
    color = vec4(0.0, 0.0,0.7, 1.0);
  } else if (length(p) > far) {
    color = vec4(0.15, 0.15, 0.6, 0.0);
  } else {
    float nu = ((length(p) - near) / (far - near)) * ((length(p) - near) / (far - near));
    color = nu * vec4(0.15, 0.15, 0.6, 0.0) + (1-nu) * vec4(0.0, 0.0,0.7, 1.0);
  }

  gl_Position = projection_matrix * model_view_matrix * gl_Position;
  if (aspect_ratio < 1.0) {
    gl_Position.y *= aspect_ratio;

  } else {
    gl_Position.x /= aspect_ratio;
  }
}