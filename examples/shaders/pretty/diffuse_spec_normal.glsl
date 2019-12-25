#version 450

layout(location = 0) in vec2 v_tex_coord;
layout(location = 1) in vec3 tan_light_pos;
layout(location = 2) in vec3 tan_cam_pos;
layout(location = 3) in vec3 tan_frag_pos;
layout(location = 4) in vec3 v_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D shadow_map;
layout(set = 1, binding = 0) uniform Material {
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
  vec3 shininess;
  vec3 use_texture;
} material;

layout(set = 1, binding = 1) uniform Model {
  mat4 model;
} model;

layout(set = 2, binding = 0) uniform sampler2D diffuse_map;
layout(set = 2, binding = 1) uniform sampler2D specular_map;
layout(set = 2, binding = 2) uniform sampler2D normal_map;

layout(set = 3, binding = 0) uniform Camera {
  mat4 view;
  mat4 proj;
  vec3 pos;
} camera;

layout(set = 3, binding = 1) uniform Light {
  vec3 position;
  vec3 strength; // vec3 really means float, idk why it doesn't work
} light;

void main() {
  // only use the texture if we should
  vec4 tex_diffuse = material.use_texture.r > 0.5 ? texture(diffuse_map, v_tex_coord) : vec4(material.diffuse, 1.0);

  vec3 tex_specular = texture(specular_map, v_tex_coord).rgb;

  vec3 normal = texture(normal_map, v_tex_coord).rgb * 2.0 - 1.0;

  // ambient
  vec3 ambient = tex_diffuse.rgb * 0.01;

  // diffuse
  vec3 light_dir = normalize(tan_light_pos - tan_frag_pos);

  float diff = max(dot(normal, light_dir), 0.0);
  vec3 diffuse = diff * tex_diffuse.rgb;

  // specular
  vec3 view_dir = normalize(tan_cam_pos - tan_frag_pos);
  vec3 halfway_dir = normalize(light_dir + view_dir);
  float spec = pow(max(dot(normal, halfway_dir), 0.0), 32.0);
  vec3 specular = vec3(clamp(0.2 * spec, 0.0, 0.5));

  // result
  float dist = length(tan_light_pos - tan_frag_pos);

  vec3 result = ambient + (diffuse + specular) * light.strength.r / (dist * dist / 2000.0);

  vec3 corrected = pow(result, vec3(1/2.2));

  f_color = vec4(corrected, 1.0);
}
