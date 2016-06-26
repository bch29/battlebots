#version 140

in vec3 v_normal;
in vec4 v_color;
out vec4 f_color;

void main() {
  f_color = v_color;
}