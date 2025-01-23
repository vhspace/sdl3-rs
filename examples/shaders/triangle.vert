#version 440
layout (location = 0) out vec4 v_color;
// glslc triangle.vert -o triangle.vert.spv
void main(void) {
	if(gl_VertexIndex == 0) {
		gl_Position = vec4(-1.f, -1.f, 0.f, 1.f);
		v_color = vec4(1.f, 0.f, 0.f, 1.f);
	} else if(gl_VertexIndex == 1) {
		gl_Position = vec4(1.0f, -1.0f, 0.f, 1.f);
		v_color = vec4(0.0f, 1.0f, 0.f, 1.f);
	} else if(gl_VertexIndex == 2) {
		gl_Position = vec4(0.0f, 1.0f, 0.f, 1.f);
		v_color = vec4(0.0f, 0.0f, 1.f, 1.f);
	}
}
