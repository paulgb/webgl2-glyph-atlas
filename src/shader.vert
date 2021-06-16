#version 300 es

in vec2 a_position;
in vec2 a_tex_coord;

out vec2 v_tex_coord;

void main() {
    gl_Position = vec4(a_position, 0., 1.);
    v_tex_coord = a_tex_coord;
}
