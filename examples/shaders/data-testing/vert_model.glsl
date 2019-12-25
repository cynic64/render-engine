#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;
layout(location = 0) out vec3 v_color;

layout(set = 0, binding = 0) uniform Model {
  mat4 model;
} model;

void main() {
  v_color = color;
  vec2 pos = vec2(vec4(position.xy, 0.0, 1.0) * model.model);
  gl_Position = vec4(pos, 0.0, 1.0);
}
