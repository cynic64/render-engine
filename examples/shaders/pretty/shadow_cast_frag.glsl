#version 450

layout(location = 0) in vec3 v_pos;

layout(set = 3, binding = 0) uniform Light {
  vec3 position;
  vec3 strength;
} light;

void main() {
  float light_dist = length(v_pos - light.position);

  // map to 0, 1 by dividing by far plane
  light_dist /= 250.0;

  gl_FragDepth = light_dist;
}
