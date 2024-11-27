#version 450
layout( quads,equal_spacing,ccw) in;

in vec3 ts_color[];

out vec3 color;

uniform float anim_time;

uniform float aspect_ratio;

uniform float seed;

uniform mat4 model_view_matrix;
mat4 projection_matrix = mat4(
			      250.00000, 0.00000, 0.00000, 0.00000,
			      0.00000, 250.00000, 0.00000, 0.00000,
			      0.00000, 0.00000, -1.02020, -10.10101,
			      0.00000, 0.00000, -1.00000, 0.00000);

float freq = 0.5;

float wavelenght = 0.1;

#define PI  3.1415926535897932384626433832795

struct Wave {
    float freq;
    float wave_l;
    float ampl;
    float x_weight;
};
vec4 compute_y_sin_wave(Wave wave, vec3 p ) {
    return sin((anim_time * wave.freq + (p.x * wave.x_weight + p.z * sqrt(1.0 - pow(wave.x_weight, 2) )) / wave.wave_l) * 2.0 * PI) 
    * vec4(0.0, wave.ampl, 0.0, 0.0);
}


vec4 compute_y_cos_x_wave(Wave wave, vec3 p ) {
    return sin((anim_time * wave.freq + (p.x * wave.x_weight ) / wave.wave_l) * 2.0 * PI) 
    * ((wave.x_weight  / wave.wave_l) * 2.0 * PI)
    * vec4(0.0, wave.ampl, 0.0, 0.0);
}

vec4 compute_y_cos_z_wave(Wave wave, vec3 p ) {
    return sin((anim_time * wave.freq + (p.z *sqrt(1.0 - pow(wave.x_weight, 2))) / wave.wave_l) * 2.0 * PI) 
    * (((p.z * sqrt(1.0 - pow(wave.x_weight, 2))) / wave.wave_l) * 2.0 * PI) 
    * vec4(0.0, wave.ampl, 0.0, 0.0);
}

vec3 normal(Wave wave, vec3 p) {
    float y_x = compute_y_cos_x_wave(wave, p ).y;
    float y_z = compute_y_cos_z_wave(wave, p ).y;
    return normalize(-cross(vec3(1.0, y_x, 0.0), vec3(0.0,y_z, 1.0)));
}

Wave make_wave(float freq, float wave_l, float ampl, float x_weight) {
    Wave res;
    res.freq = freq;
    res.wave_l = wave_l;
    res.ampl = ampl;
    res.x_weight = x_weight;
    return res;
}

void main(){
  vec4 p1 = mix(gl_in[0].gl_Position,gl_in[1].gl_Position,gl_TessCoord.x);
  vec4 p2 = mix(gl_in[3].gl_Position,gl_in[2].gl_Position,gl_TessCoord.x);
  vec4 p = mix(p1 ,p2 ,gl_TessCoord.y);
  Wave waves[3];
  waves[0] = make_wave(0.1, 0.3, 0.02,1.0);
  waves[1] = make_wave(0.45359512, 0.09, 0.01, 0.0);
  waves[2] = make_wave(0.02, 0.4, 0.025,0.7);

  gl_Position = projection_matrix * model_view_matrix * (
      p
    + compute_y_sin_wave(waves[0], p.xyz)
    + compute_y_sin_wave(waves[1], p.xyz)
    + compute_y_sin_wave(waves[2], p.xyz)
  );
  if (aspect_ratio < 1.0) {
    gl_Position.y *= aspect_ratio;

  } else {
    gl_Position.x /= aspect_ratio;
  }
  vec3 normalWS = vec3(0.0, 0.0, 0.0);
  for (int i = 0; i < 3; ++i ) {
    normalWS += normal(waves[i], p.xyz);
  }

  color = max(dot(normalize(normalWS), normalize(vec3(1.0,1.0,1.0))), 0.0) * mix(mix(ts_color[0], ts_color[1], gl_TessCoord.x), mix(ts_color[2], ts_color[3], gl_TessCoord.x), gl_TessCoord.y);
}
