#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Model {
    mat4 model;
} model;

layout(set = 1, binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
} camera;

void main() {
     gl_Position = camera.proj * camera.view * model.model * vec4(position, 1.0);
}
