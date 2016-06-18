#version 100

precision mediump float;
precision mediump int;

varying vec4 v_Color;

void main() {
    gl_FragColor = v_Color;
}
