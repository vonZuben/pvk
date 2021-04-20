#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) out uvec4 outColor;

void main() {
    outColor = uvec4(255, 0, 0, 0);
}