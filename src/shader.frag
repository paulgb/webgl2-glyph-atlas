#version 300 es
precision mediump float;

uniform sampler2D u_texture;
in vec2 v_tex_coord;
out vec4 f_color;

void main() {
    f_color = texture(u_texture, v_tex_coord);
    //f_color = vec4(f_color.rgb * f_color.a, f_color.a);
}
