#version 450

layout(location = 0) in vec2 v_pos;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D depth_map;

void main() {
  float depth = texture(depth_map, v_pos.xy).r;
  f_color = vec4(vec3(pow(depth, 20.0)), 1.0);
}
