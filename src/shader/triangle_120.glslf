#version 120

uniform sampler2D t_Tex;
varying vec4 v_Color;
varying vec2 v_Uv;

void main() {
    gl_FragColor = v_Color * texture2D(t_Tex, v_Uv);
}
