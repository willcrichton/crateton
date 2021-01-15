#version 300 es
//#version 450

// https://github.com/Shrimpey/Outlined-Diffuse-Shader-Fixed

#if __VERSION__ == 450
layout(location = 0) out vec4 o_Target;
#else
precision highp float;
out vec4 o_Target;

vec4 encodeSRGB(vec4 linearRGB_in) {
    vec3 linearRGB = linearRGB_in.rgb;
    vec3 a = 12.92 * linearRGB;
    vec3 b = 1.055 * pow(linearRGB, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linearRGB);
    return vec4(mix(a, b, c), linearRGB_in.a);
}
#endif

void main() {
    o_Target = vec4(0.0, 0.2, 0.8, 1.0);
#if __VERSION__ == 300
    o_Target = encodeSRGB(o_Target);
#endif    
}
