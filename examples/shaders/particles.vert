#version 450
layout(location = 0) out vec4 outColor;

struct Particle {
    vec2 pos;
    vec2 vel;
    vec4 color;
};

layout(set = 0, binding = 0) readonly buffer Particles {
    Particle particles[];
};

void main() {
    vec2 pos = particles[gl_VertexIndex].pos.xy;
    gl_Position = vec4(pos, 0.0, 1.0);
    gl_PointSize = 2.0;
    outColor = particles[gl_VertexIndex].color;
}
