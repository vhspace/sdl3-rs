#version 440
layout (location = 0) in vec4 v_color;
layout (location = 0) out vec4 o_frag_color;
// glslc triangle.frag -o triangle.frag.spv
void main() {
	o_frag_color = v_color;
}
