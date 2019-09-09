#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;
layout(location = 0) out vec3 v_color;
layout(location = 1) out vec3 v_normal;

/*
layout(set = 0, binding = 0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 proj;
} uniforms;
*/

void main() {
    /* mat4 modelview = uniforms.view * uniforms.model; */
    /* gl_Position = uniforms.proj * modelview * vec4(position, 1.0); */
    gl_Position = vec4(position * 0.1, 1.0);
    v_color = color;
    v_normal = normal;
}
