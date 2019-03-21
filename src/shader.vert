#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 coord;
layout(location = 3) in vec3 tangent;
layout(location = 4) in vec3 bitanget;

layout(location = 0) out vec3 FragPos;
layout(location = 1) out vec2 TexCoords;
layout(location = 2) out vec3 TangentLightPos;
layout(location = 3) out vec3 TangentViewPos;
layout(location = 4) out vec3 TangentFragPos;
layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    vec3 camera;
} uniforms;
void main() {
    FragPos = vec3(uniforms.world * vec4(position, 1.0));
    TexCoords = coord;

    mat3 normalMatrix = transpose(inverse(mat3(uniforms.world)));
    vec3 T = normalize(normalMatrix * tangent);
    vec3 N = normalize(normalMatrix * normal);
    vec3 B = cross(N,T);
    mat3 TBN = transpose(mat3(T,B,N));

    TangentLightPos = TBN * uniforms.camera;
    TangentViewPos = TBN * uniforms.camera;
    TangentFragPos = TBN * FragPos;

    gl_Position = uniforms.proj * uniforms.view * uniforms.world * vec4(position, 1.0);
    gl_Position.y = -gl_Position.y;
    gl_Position.z = (gl_Position.z + gl_Position.w)/2.0;
}