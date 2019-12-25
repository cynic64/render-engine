#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;

layout(location = 0) out vec3 v_normal;

layout(set = 0, binding = 0) uniform Model {
  mat4 model;
} model;

layout(set = 1, binding = 0) uniform Camera {
  mat4 view;
  mat4 proj;
} camera;

void main() {
     v_normal = normal;
     gl_Position = camera.proj * camera.view * model.model * vec4(position, 1.0);
}
