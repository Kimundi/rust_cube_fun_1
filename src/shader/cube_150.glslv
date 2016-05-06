#version 150 core

in ivec4 a_Pos;
in ivec2 a_TexCoord;
in vec3 a_Translate;
out vec2 v_TexCoord;

uniform mat4 u_Transform;

void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = u_Transform * (a_Pos + vec4(a_Translate, 0));
}
