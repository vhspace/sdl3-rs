#version 450

// Final color of the pixel
layout (location = 0) out vec4 final_color;

// Color from our vertex shader
layout (location = 0) in vec3 frag_color;

void main() {
	final_color = vec4(frag_color, 1.0);
}