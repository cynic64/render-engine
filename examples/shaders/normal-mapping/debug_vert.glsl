#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 v_pos;
layout(location = 1) out vec3 v_color;

layout(set = 0, binding = 0) uniform Model {
    mat4 model;
} model;

layout(set = 1, binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
    vec3 pos;
} camera;

void main() {
     v_pos = vec3(model.model * vec4(position, 1.0));
     v_color = color;
     gl_Position = camera.proj * camera.view * vec4(v_pos, 1.0);
}
