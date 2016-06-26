#version 100

attribute vec2 a_Pos;
attribute vec2 a_Uv;
attribute vec3 a_Color;
varying vec4 v_Color;
varying vec2 v_Uv;

void main() {
    v_Color = vec4(a_Color, 1.0);
    v_Uv = a_Uv;
    gl_Position = vec4(a_Pos, 0.0, 1.0);
}