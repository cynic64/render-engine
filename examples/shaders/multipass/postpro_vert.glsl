#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 v_tex_coords;

void main() {
  v_tex_coords = position * 0.5 + 0.5;
  gl_Position = vec4(position, 0.0, 1.0);
}
