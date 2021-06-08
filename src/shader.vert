#version 300 es

in vec2 a_upper_left;
in vec2 a_lower_right;
in vec2 a_tex_upper_left;
in vec2 a_tex_lower_right;

out vec2 v_tex_coord;

void main() {
    switch (gl_VertexID) {
        case 0:
        gl_Position = vec4(a_upper_left.xy, 0., 1.);
        v_tex_coord = a_tex_upper_left;
        break;
        case 1:
        case 3:
        gl_Position = vec4(a_lower_right.x, a_upper_left.y, 0., 1.);
        v_tex_coord = vec2(a_tex_lower_right.x, a_tex_upper_left.y);
        break;
        case 2:
        case 4:
        gl_Position = vec4(a_upper_left.x, a_lower_right.y, 0., 1.);
        v_tex_coord = vec2(a_tex_upper_left.x, a_tex_lower_right.y);
        break;
        case 5:
        gl_Position = vec4(a_lower_right.xy, 0., 1.);
        v_tex_coord = a_tex_lower_right;
    }
}
