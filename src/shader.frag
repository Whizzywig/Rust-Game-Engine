#version 450
layout(location = 0) in vec3 FragPos;
layout(location = 1) in vec2 TexCoords;
layout(location = 2) in vec3 TangentLightPos;
layout(location = 3) in vec3 TangentViewPos;
layout(location = 4) in vec3 TangentFragPos;
layout(location = 5) in mat3 TBN;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 2) uniform sampler2D normalMap;
layout(set = 0, binding = 3) uniform sampler2D metallicMap;
layout(set = 0, binding = 4) uniform sampler2D roughnessMap;
layout(set = 0, binding = 5) uniform sampler2D aoMap;

const float PI = 3.14159265359;

float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

void main() {
    vec3 normal = texture(normalMap, TexCoords).rgb;
    normal = normal*2.0 - 1.0;
    vec3 N = normalize(normal * TBN);
    normal = normalize(normal);

    float metallic  = texture(metallicMap, TexCoords).r;
    float roughness = texture(roughnessMap, TexCoords).r;
    float ao        = texture(aoMap, TexCoords).r;
    vec3 color = texture(tex, TexCoords).rgb;

    //vec3 V = normalize(camPos - WorldPos);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, color, metallic);

    vec3 Lo = vec3(0.0);

    vec3 lightDir = normalize(TangentLightPos - TangentFragPos);
    vec3 viewDir = normalize(TangentViewPos - TangentFragPos);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 halfwayDir = normalize(lightDir + viewDir);

    float distance = length(TangentLightPos - TangentFragPos);
    float attenuation = 1.0/(distance * distance);
    vec3 radiance = vec3(1.0, 1.0, 1.0) * attenuation;

    float NDF = DistributionGGX(N, halfwayDir, roughness);
    float G = GeometrySmith(N, V, L, roughness);
    vec3 F = fresnelSchlick(max(dot(halfwayDir, V), 0.0), F0);

    vec3 nominator = NDF * G * F;
    float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L),0.0) + 0.001;
    vec3 specular = nominator / denominator;

    vec3 kS =F;

    vec3 kD = vec3(1.0) - kS;

    float NdotL = max(dot(N, L),0.0);
    Lo += (kD * albedo/ PI + specular) * radiance * NdotL;

    vec3 ambient = color * ao * vec(0.03);

    color = (ambient + Lo);
    color = color/(color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));

    f_color = vec4(color, 1.0);
}