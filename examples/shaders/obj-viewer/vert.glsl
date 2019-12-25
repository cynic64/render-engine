#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec3 tangent;

layout(location = 0) out vec2 v_tex_coord;
layout(location = 1) out vec3 tan_light_dir;
layout(location = 2) out vec3 tan_cam_pos;
layout(location = 3) out vec3 tan_frag_pos;

layout(set = 0, binding = 0) uniform Material {
  vec4 ambient;
  vec4 diffuse;
  vec4 specular;
  vec4 shininess;
  vec4 use_texture;
} material;

layout(set = 0, binding = 1) uniform Model {
  mat4 model;
} model;

layout(set = 1, binding = 0) uniform sampler2D diffuse_map;
layout(set = 1, binding = 1) uniform sampler2D specular_map;
layout(set = 1, binding = 2) uniform sampler2D normal_map;

layout(set = 2, binding = 0) uniform Camera {
  mat4 view;
  mat4 proj;
  vec3 pos;
} camera;

layout(set = 2, binding = 1) uniform Light {
  vec3 direction;
  vec3 strength; // vec3 really means float, idk why it doesn't work
} light;

void main() {
  v_tex_coord = tex_coord;
  vec3 pos = vec3(model.model * vec4(position, 1.0));
  gl_Position = camera.proj * camera.view * vec4(pos, 1.0);

  vec3 bitangent = cross(tangent, normal);
  mat3 TBN = transpose(mat3(tangent, bitangent, normal));
  tan_light_dir = normalize(TBN * light.direction);
  tan_cam_pos = TBN * camera.pos;
  tan_frag_pos = TBN * pos;
}