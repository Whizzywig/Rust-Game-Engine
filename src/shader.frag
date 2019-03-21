#version 450
layout(location = 0) in vec3 FragPos;
layout(location = 1) in vec2 TexCoords;
layout(location = 2) in vec3 TangentLightPos;
layout(location = 3) in vec3 TangentViewPos;
layout(location = 4) in vec3 TangentFragPos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 2) uniform sampler2D normalmap;
void main() {
    vec3 normal = texture(normalmap, TexCoords).rgb;
    normal = normalize(normal*2.0 - 1.0);

    vec3 color = texture(tex, TexCoords).rgb;
    vec3 ambient = color *0.1;

    vec3 lightDir = normalize(TangentLightPos - TangentFragPos);

    float diff = max(dot(lightDir, normal),0.0);
    vec3 diffuse = diff * color;
    vec3 viewDir = normalize(TangentViewPos - TangentFragPos);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir),0.0),32.0);
    vec3 specular = vec3(0.2) * spec;

    f_color = vec4(ambient + diffuse + specular, 1.0);
}