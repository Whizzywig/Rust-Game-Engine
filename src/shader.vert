#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 coord;
layout(location = 3) in vec3 tangent;
layout(location = 4) in vec3 bitangent;

layout(location = 0) out vec2 TexCoords;
layout(location = 1) out vec3 WorldPos;
layout(location = 2) out vec3 Normal;
layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    vec3 camera;
} uniforms;
void main() {
    TexCoords = coord;
    WorldPos = vec3(uniforms.world * vec4(position, 1.0));
    Normal = mat3(uniforms.world) * normal;

    gl_Position = uniforms.proj * uniforms.view * uniforms.world * vec4(position, 1.0);
    gl_Position.y = -gl_Position.y;
    gl_Position.z = (gl_Position.z + gl_Position.w)/2.0;
}