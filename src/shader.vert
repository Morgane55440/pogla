#version 450

in vec3 position;

out vec3 color;

vec3 light_color = vec3(1.0, 1.0, 1.0);
vec3 light_position = vec3(-10.0, -10.0, -50.0);

vec3 object_color = vec3(0.0, 1.0, 1.0);


void main() {
    vec3 normalSmooth = vec3(0.0, 1.0, 0.0);
  gl_Position = vec4(position, 1.0);

  float c = clamp(dot(normalize(position - light_position), normalSmooth), 0.0, 1.0);
  color = object_color * c;

}