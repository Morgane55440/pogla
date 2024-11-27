#version 450
layout( quads,equal_spacing,ccw) in;


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


vec3 mod289(vec3 x) {
  return x - floor(x * (1.0 / 289.0)) * 289.0;
}

vec2 mod289(vec2 x) {
  return x - floor(x * (1.0 / 289.0)) * 289.0;
}

vec3 permute(vec3 x) {
  return mod289(((x*34.0)+10.0)*x);
}

float snoise(vec2 v, float ampl)
  {
    v = v + vec2(15.2654 * seed, -62.34584 * seed);
  const vec4 C = vec4(0.211324865405187,  // (3.0-sqrt(3.0))/6.0
                      0.366025403784439,  // 0.5*(sqrt(3.0)-1.0)
                     -0.577350269189626,  // -1.0 + 2.0 * C.x
                      0.024390243902439); // 1.0 / 41.0
// First corner
  vec2 i  = floor(v + dot(v, C.yy) );
  vec2 x0 = v -   i + dot(i, C.xx);

// Other corners
  vec2 i1;
  //i1.x = step( x0.y, x0.x ); // x0.x > x0.y ? 1.0 : 0.0
  //i1.y = 1.0 - i1.x;
  i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
  // x0 = x0 - 0.0 + 0.0 * C.xx ;
  // x1 = x0 - i1 + 1.0 * C.xx ;
  // x2 = x0 - 1.0 + 2.0 * C.xx ;
  vec4 x12 = x0.xyxy + C.xxzz;
  x12.xy -= i1;

// Permutations
  i = mod289(i); // Avoid truncation effects in permutation
  vec3 p = permute( permute( i.y + vec3(0.0, i1.y, 1.0 ))
		+ i.x + vec3(0.0, i1.x, 1.0 ));

  vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), 0.0);
  m = m*m ;
  m = m*m ;

// Gradients: 41 points uniformly over a line, mapped onto a diamond.
// The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)

  vec3 x = 2.0 * fract(p * C.www) - 1.0;
  vec3 h = abs(x) - 0.5;
  vec3 ox = floor(x + 0.5);
  vec3 a0 = x - ox;

// Normalise gradients implicitly by scaling m
// Approximation of: m *= inversesqrt( a0*a0 + h*h );
  m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );

// Compute final noise value at P
  vec3 g;
  g.x  = a0.x  * x0.x  + h.x  * x0.y;
  g.yz = a0.yz * x12.xz + h.yz * x12.yw;
  return ampl * dot(m, g);
}



float falloff_radius = 1.0;

float fall_off(vec2 v) {
    float dx = (abs(v.x) / falloff_radius);
    float dz = (abs(v.y) / falloff_radius);
    return 1.0 - min(1.0, (dx * dx * dx + dz * dz * dz));
}

vec3 snoise_normal(vec2 v, float ampl)
{
    float epsilon = 0.0001;

    float dx = (snoise(v + vec2(epsilon, 0.0), ampl) - snoise(v - vec2(epsilon, 0.0), ampl)) *  fall_off(v);
    float dz = (snoise(v + vec2(0.0, epsilon), ampl) - snoise(v - vec2(0.0, epsilon), ampl)) * fall_off(v);

    return normalize(vec3(-dx/epsilon, 1.0, -dz/epsilon));
}

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

float lighten_up(float f) {
    return min(1.0, 4.0 * f);
}

void main(){
  vec4 p1 = mix(gl_in[0].gl_Position,gl_in[1].gl_Position,gl_TessCoord.x);
  vec4 p2 = mix(gl_in[3].gl_Position,gl_in[2].gl_Position,gl_TessCoord.x);
  vec4 p = mix(p1 ,p2 ,gl_TessCoord.y);
  Wave waves[3];
  waves[0] = make_wave(0.1, 0.3, 0.02,1.0);
  waves[1] = make_wave(0.45359512, 0.09, 0.01, 0.0);
  waves[2] = make_wave(0.02, 0.4, 0.025,0.7);

  gl_Position =  (
      p + (vec4(0.0,snoise(p.xz, 25), 0.0,0.0) + 0.1) * fall_off(p.xz)
    //+ compute_y_sin_wave(waves[0], p.xyz)
    //+ compute_y_sin_wave(waves[1], p.xyz)
    //+ compute_y_sin_wave(waves[2], p.xyz)
  );
  vec3 normalWS = vec3(0.0, 0.0, 0.0);

  if (gl_Position.y <= 0.0) {
    gl_Position.y = 0.0;
    normalWS = vec3(0.0,1.0,0.0);
  } else {
    for (int i = 0; i < 3; ++i ) {
        normalWS += normal(waves[i], p.xyz);
    }

  }


    normalWS = snoise_normal(p.xz, 20);
    vec3 raw_color = vec3(0.0, 0.6,0.0);
    if (gl_Position.y <= 0.01) {
        raw_color = vec3(0.5, 0.5, 0.0);
    }
    if (gl_Position.y <= 0.0001) {
        raw_color = vec3(0.0, 0.0,0.8);
    }
    if (gl_Position.y >= 0.18) {
        raw_color = vec3(0.3, 0.3, 0.3);
    }
    color = max(dot(normalize(normalWS), normalize(vec3(1.0,4.0,1.0))), 0.0) * raw_color;
  
    if (gl_Position.y <= 0.0001) {
        color = vec3(0.0, 0.0,0.7);
    }

  gl_Position = projection_matrix * model_view_matrix * gl_Position;
  if (aspect_ratio < 1.0) {
    gl_Position.y *= aspect_ratio;

  } else {
    gl_Position.x /= aspect_ratio;
  }
}
