#version 450

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_tex_coord;
layout(location = 3) in vec3 tan_light_pos;
layout(location = 4) in vec3 tan_cam_pos;
layout(location = 5) in vec3 tan_frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform Model {
  mat4 model;
} model;

layout(set = 0, binding = 1) uniform Material {
  float shininess;
} material;

layout(set = 1, binding = 0) uniform Camera {
  mat4 view;
  mat4 proj;
  vec3 pos;
} camera;

layout(set = 2, binding = 0) uniform Light {
  vec3 pos;
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
} light;

layout(set = 3, binding = 0) uniform sampler2D diffuse_texture;
layout(set = 3, binding = 1) uniform sampler2D specular_texture;
layout(set = 3, binding = 2) uniform sampler2D normal_texture;

void main() {
    vec3 tex_diffuse = texture(diffuse_texture, v_tex_coord).rgb;
    vec3 tex_specular = texture(specular_texture, v_tex_coord).rgb;

    vec3 normal = normalize(texture(normal_texture, v_tex_coord).rgb * 2.0 - 1.0);

    // ambient
    vec3 ambient = tex_diffuse * light.ambient;

    // diffuse
    vec3 light_dir = normalize(tan_light_pos - tan_frag_pos);

    float diff = max(dot(normal, light_dir), 0.0);
    vec3 diffuse = light.diffuse * (diff * tex_diffuse);

    // specular
    vec3 view_dir = normalize(tan_cam_pos - tan_frag_pos);
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), material.shininess);
    vec3 specular = light.specular * (spec * tex_specular);

    // result
    vec3 result = ambient + diffuse + specular;

    // gamma correction
    float gamma = 2.2;
    result.rgb = pow(result.rgb, vec3(1.0/gamma));

    f_color = vec4(result, 1.0);
}
