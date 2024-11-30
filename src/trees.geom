#version 450

layout(points) in;
layout(triangle_strip, max_vertices=120) out; 

out vec3 color;
uniform mat4 model_view_matrix;


uniform float daytime;

#define PI  3.1415926535897932384626433832795

uniform float aspect_ratio;

mat4 projection_matrix = mat4(
			      250.00000, 0.00000, 0.00000, 0.00000,
			      0.00000, 250.00000, 0.00000, 0.00000,
			      0.00000, 0.00000, -1.02020, -10.10101,
			      0.00000, 0.00000, -1.00000, 0.00000);

void main() {
    vec4 inputPoint = gl_in[0].gl_Position; 
    
    vec3 sun_pos = vec3(20.0, -200 * cos(daytime * PI / 12), 200 * sin(daytime * PI / 12));

    if (inputPoint.y > 0.0) {
        float height = 0.05 + mod(gl_in[0].gl_Position.y * PI + 17.245, 0.045);
        float radius = 0.005; 
        vec4 apex = gl_in[0].gl_Position + vec4(0.0, height, 0.0, 0.0);

        vec4 base[8];
        vec3 normals[8];
        for (int i = 0; i < 8; i++) {
            float angle = 2.0 * PI * float(i) / 8.0;
            base[i] = gl_in[0].gl_Position + vec4(
                radius * cos(angle),
                0.0,                 
                radius * sin(angle), 
                0.0                  
            );
            normals[i] = normalize(vec3(
                radius * cos(angle),
                0.0,                 
                radius * sin(angle)               
            ));

        }

        for (int i = 0; i < 8; i++) {
            vec3 v0 = vec3(base[i]);
            vec3 v1 = vec3(apex);                       
            vec3 v2 = vec3(base[(i + 1) % 8]);
            vec3 center = (v0 + v1 + v2) / 3.0;

            gl_Position = projection_matrix * model_view_matrix * vec4(v0,1.0);
            if (aspect_ratio < 1.0) {
                gl_Position.y *= aspect_ratio;

            } else {
                gl_Position.x /= aspect_ratio;
            }
            color = min(max(dot(normals[i],normalize(sun_pos - center)),0.0) + 0.1, 1.0) * vec3(0.545, 0.271, 0.075);
            EmitVertex();

            gl_Position = projection_matrix * model_view_matrix * vec4(v1,1.0);
            if (aspect_ratio < 1.0) {
                gl_Position.y *= aspect_ratio;

            } else {
                gl_Position.x /= aspect_ratio;
            }
            color = min(max(dot(vec3(0.0, 1.0, 0.0),normalize(sun_pos - center)),0.0) + 0.1, 1.0) * vec3(0.545, 0.271, 0.075);;
            EmitVertex();

            gl_Position = projection_matrix * model_view_matrix * vec4(v2,1.0);
            if (aspect_ratio < 1.0) {
                gl_Position.y *= aspect_ratio;

            } else {
                gl_Position.x /= aspect_ratio;
            }
            color = min(max(dot(normals[(i + 1) % 8],normalize(sun_pos - center)),0.0) + 0.1, 1.0) * vec3(0.545, 0.271, 0.075);;
            EmitVertex();

            EndPrimitive(); 
        }

        float newbase = height - 0.02;
        radius = 0.015;
        while (newbase > 0.01) {
            for (int i = 0; i < 8; i++) {
                float angle = 2.0 * PI * float(i) / 8.0;
                base[i] = gl_in[0].gl_Position + vec4(
                    radius * cos(angle),
                    newbase,                 
                    radius * sin(angle), 
                    0.0                  
                );
                normals[i] = normalize(vec3(
                    cos(angle),
                    0.0,                 
                    sin(angle)               
                ));
            }

            for (int i = 0; i < 8; i++) {
                vec3 v0 = vec3(base[i]);
                vec3 v1 = vec3(apex);                       
                vec3 v2 = vec3(base[(i + 1) % 8]);
                vec3 center = (v0 + v1 + v2) / 3.0;

                gl_Position = projection_matrix * model_view_matrix * vec4(v0,1.0);
                if (aspect_ratio < 1.0) {
                    gl_Position.y *= aspect_ratio;

                } else {
                    gl_Position.x /= aspect_ratio;
                }
                color = min(max(dot(normals[i],normalize(sun_pos - center)),0.0) + 0.1, 1.0) * vec3(0.0, 0.5, 0.0);
                EmitVertex();

                gl_Position = projection_matrix * model_view_matrix * vec4(v1,1.0);
                if (aspect_ratio < 1.0) {
                    gl_Position.y *= aspect_ratio;

                } else {
                    gl_Position.x /= aspect_ratio;
                }
                color = min(max(dot(vec3(0.0, 1.0, 0.0),normalize(sun_pos - center)),0.0) + 0.1, 1.0) * vec3(0.0, 0.5, 0.0);
                EmitVertex();

                gl_Position = projection_matrix * model_view_matrix * vec4(v2,1.0);
                if (aspect_ratio < 1.0) {
                    gl_Position.y *= aspect_ratio;

                } else {
                    gl_Position.x /= aspect_ratio;
                }
                color = min(max(dot(normals[(i + 1) % 8],normalize(sun_pos - center)),0.0) + 0.1, 1.0) * vec3(0.0, 0.5, 0.0);
                EmitVertex();

                EndPrimitive(); 
            }
            newbase -= 0.02;
            radius += 0.004;
        }
    }
}