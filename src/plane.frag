#version 450

in vec4 color;

layout(location=0) out vec4 output_color;

void main() {
  output_color = color;
}