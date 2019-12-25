#version 450

layout(location = 0) in vec2 v_pos;

layout(set = 0, binding = 0) uniform sampler2D depth_map;

void main() {
  float depth = 0.0;
  float radius = 0.0002;
  for (int x = -2; x <= 2; x++) {
    for (int y = -2; y <= 2; y++) {
      vec2 tex_coords = v_pos.xy + vec2(x * radius, y * radius);
      float sample_depth = texture(depth_map, tex_coords).r;
      depth += sample_depth;
    }
  }
  gl_FragDepth = depth / 25.0;
}
