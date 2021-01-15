#version 300 es
//#version 450

// https://github.com/Shrimpey/Outlined-Diffuse-Shader-Fixed

#if __VERSION__ == 450
layout(location = 0) in vec3 Vertex_Position;
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
#else
precision highp float;
in vec3 Vertex_Position;
layout(std140) uniform Camera {
    mat4 ViewProj;
};
layout(std140) uniform Transform { // set = 1, binding = 0
    mat4 Model;
};
#endif 

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position * 1.05, 1.0);
}