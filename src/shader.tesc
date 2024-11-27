#version 450

layout( vertices = 4 ) out; 

in vec3 color[];

out vec3 ts_color[];

uniform int tess_level;


void main() {
   gl_TessLevelOuter[0] = tess_level;
   gl_TessLevelOuter[1] = tess_level;
   gl_TessLevelOuter[2] = tess_level;
   gl_TessLevelOuter[3] = tess_level;
   gl_TessLevelInner[0] = tess_level;
   gl_TessLevelInner[1] = tess_level;
   gl_out[gl_InvocationID].gl_Position=gl_in[gl_InvocationID].gl_Position;
   ts_color[gl_InvocationID] = color[gl_InvocationID];

}
