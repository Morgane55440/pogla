#version 450
layout( quads,equal_spacing,ccw) in;


#define NOISE_NB 3

out vec4 color;

uniform float anim_time;

uniform float aspect_ratio;

uniform float seed;

uniform float daytime;

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

struct Noise {
    float freq;
    float ampl;
};

float compute_noise(vec2 p, Noise noise) {
    return snoise(p * noise.freq, noise.ampl);
}




float falloff_radius = 1.0;

float fall_off(vec2 v) {
    float dx = (abs(v.x) / falloff_radius);
    float dz = (abs(v.y) / falloff_radius);
    return 1.0 - min(1.0, (dx * dx  + dz * dz ));
}

vec3 snoise_normal(vec2 v, Noise noises[NOISE_NB])
{
    float epsilon = 0.0001;
    float dx = 0;
    float dz = 0;
    for (int i = 0; i < NOISE_NB; ++i) {
        dx += (compute_noise(v + vec2(epsilon, 0.0), noises[i]) - compute_noise(v - vec2(epsilon, 0.0), noises[i])) *  fall_off(v);
        dz += (compute_noise(v + vec2(0.0, epsilon), noises[i]) - compute_noise(v - vec2(0.0, epsilon), noises[i])) *  fall_off(v);
    }
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

float compute_noises(vec2 v, Noise noises[NOISE_NB]){
    float res = 0.0;
    for (int i = 0; i < NOISE_NB; ++i) {
        res += compute_noise(v , noises[i]);
    }
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

  Noise noises[NOISE_NB];

  noises[0].freq = 1.0;
  noises[0].ampl = 25;
  
  float base_noise = compute_noise(p.xz, noises[0]);

  noises[1].freq = 5.0;
  noises[1].ampl = base_noise * base_noise * 200;

  
  noises[2].freq = 25.0;
  noises[2].ampl = base_noise * base_noise * 40;

  if (base_noise * fall_off(p.xz) <= 0.175) {
    float desent = 1.0 / ((0.175 - base_noise * fall_off(p.xz) )  * 5.0 + 1.0);
    noises[2].ampl *= desent * desent * desent;
  }


  gl_Position =  (
      p + (
        0.1 + vec4(0.0,compute_noises(p.xz, noises), 0.0,0.0)
      ) 
      
      * fall_off(p.xz)
    //+ compute_y_sin_wave(waves[0], p.xyz)
    //+ compute_y_sin_wave(waves[1], p.xyz)
    //+ compute_y_sin_wave(waves[2], p.xyz)
  );
  vec3 normalWS = vec3(0.0, 0.0, 0.0);


    normalWS = snoise_normal(p.xz, noises);
    vec4 raw_color = vec4(0.0, 0.6,0.0, 1.0);


    if (gl_Position.y <= 0.01) {
        raw_color = vec4(0.5, 0.5, 0.0, 1.0);
    } else {
        Noise choice_noise;
        choice_noise.freq = 8;
        choice_noise.ampl = 2;
        float y = gl_Position.y + compute_noise(p.xz, choice_noise);
        if (y >= 0.16) {
            raw_color = vec4(0.3, 0.3, 0.3, 1.0);
        } else {
            noises[0].freq = 0.6;
            if (compute_noises(p.xz + 10.0, noises) > 0.07) {
                raw_color = vec4(0.0, 0.35,0.0, 1.0);
            }
        }

    }

    vec3 sun_pos = vec3(20.0, -200 * cos(daytime * PI / 12), 200 * sin(daytime * PI / 12));
    color = max(dot(normalize(normalWS), normalize(sun_pos - p.xyz)), 0.1) * raw_color;
    color.w = 1.0;
  
    if (gl_Position.y <= 0.0001) {
        color = vec4(0.5, 0.5,0.0, 0.0);
    }

  gl_Position = projection_matrix * model_view_matrix * gl_Position;
  if (aspect_ratio < 1.0) {
    gl_Position.y *= aspect_ratio;

  } else {
    gl_Position.x /= aspect_ratio;
  }
}
