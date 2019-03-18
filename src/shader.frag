#version 450
layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 texcoord;
layout(location = 0) out vec4 f_color;
const vec3 LIGHT = vec3(0.0, 0.0, 1.0);
layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 2) uniform sampler2D normalmap;
void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    //vec3 dark_color = vec3(0.6, 0.0, 0.0);
    //vec3 regular_color = vec3(1.0, 0.0, 0.0);

    vec3 rgb_normal = (v_normal * 0.5 + 0.);
    vec3 normal = texture(normalmap, texcoord).rgb;
    normal = normalize(normal*2.0 -1.0);

    //f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);

    vec3 color = texture(tex,texcoord).rgb;
    vec3 ambient = color * 0.5;

    float diff = max(dot(rgb_normal, normal), 0.0);

    //f_color.rgb = (diff * color) + ambient;
    f_color.rgb = (brightness * color) + ambient;
    f_color.a = 1.0;
}