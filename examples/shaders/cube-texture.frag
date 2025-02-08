#version 450

// Texture sampler
layout (set = 2, binding = 0) uniform sampler2D tex_sampler;

// Texture coordinates from the vertex shader
layout (location = 0) in vec2 tex_coord;

// Color from our vertex shader
layout (location = 1) in vec3 frag_color;

// Final color of the pixel
layout (location = 0) out vec4 final_color;

void main() {
	final_color = texture(tex_sampler, tex_coord) * vec4(frag_color, 1.0);
}