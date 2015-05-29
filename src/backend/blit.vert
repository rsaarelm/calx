#version 150 core

attribute vec2 pos;
attribute vec2 tex_coord;

out vec2 v_tex_coord;

void main() {
    v_tex_coord = tex_coord;
    gl_Position = vec4(pos, 0.0, 1.0);
}
