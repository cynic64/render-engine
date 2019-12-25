#version 450

layout(location = 0) in vec2 v_tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D image;

void main() {
  f_color = texture(image, v_tex_coords) * 0.5 + 0.5;
}
