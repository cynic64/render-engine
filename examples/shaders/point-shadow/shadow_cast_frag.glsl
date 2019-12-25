#version 450

layout(location = 0) in vec3 v_pos;

void main() {
  // light pos is fixed at center
  float light_dist = length(v_pos);

  // map to 0, 1 by dividing by far plane
  light_dist /= 250.0;

  gl_FragDepth = light_dist;
}
