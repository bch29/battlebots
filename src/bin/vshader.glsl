#version 140

uniform mat4 global_matrix;
in vec2 position;
in vec4 color;
in mat4 instance_matrix;

out vec3 v_position;
out vec4 v_color;

void main() {
  v_position = vec3(position, 1.0);
  v_color = color;

  gl_Position = global_matrix * instance_matrix * vec4(v_position, 1.0);
}