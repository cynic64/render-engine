#version 450

layout(location = 0) in vec2 v_tex_coord;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D in_tex;

void main() {
  f_color = texture(in_tex, v_tex_coord);
}
