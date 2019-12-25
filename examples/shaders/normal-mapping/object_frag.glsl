#version 450

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec2 v_tex_coord;
layout(location = 2) in vec3 v_normal;
layout(location = 3) in vec3 tan_light_pos;
layout(location = 4) in vec3 tan_cam_pos;
layout(location = 5) in vec3 tan_frag_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform Model {
  mat4 model;
} model;

layout(set = 1, binding = 0) uniform Camera {
  mat4 view;
  mat4 proj;
  vec3 pos;
} camera;

layout(set = 2, binding = 0) uniform Light {
  vec3 pos;
} light;

layout(set = 3, binding = 0) uniform sampler2D normal_tex;

void main() {
    vec3 m_ambient = vec3(0.2);
    vec3 m_diffuse = vec3(0.7);

    vec3 normal = texture(normal_tex, v_tex_coord).rgb;
    normal = normalize(normal * 2.0 - 1.0);

    // ambient
    vec3 ambient = m_diffuse * m_ambient;

    // diffuse
    vec3 light_dir = normalize(tan_light_pos - tan_frag_pos);
    float diff = max(dot(normal, light_dir), 0.0);
    vec3 diffuse = diff * m_diffuse;

    // specular
    vec3 view_dir = normalize(tan_cam_pos - tan_frag_pos);
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    vec3 specular = vec3(spec) * 0.3;

    // result
    vec3 result = ambient + diffuse + specular;

    f_color = vec4(result, 1.0);
}
