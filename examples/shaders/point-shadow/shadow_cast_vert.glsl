#version 450

layout(location = 0) in vec3 position;
layout(location = 0) out vec3 v_pos;

layout(set = 0, binding = 0) uniform Model {
  mat4 model;
} model;

layout(set = 1, binding = 0) uniform Proj {
  mat4 proj;
} shadow_proj;

layout(set = 2, binding = 0) uniform View {
  mat4 view;
} shadow_view;

void main() {
  v_pos = vec3(model.model * vec4(position, 1.0));
  gl_Position = shadow_proj.proj * shadow_view.view * vec4(v_pos, 1.0);
}
