#!/bin/sh
glslc -O triangle.frag -o triangle.frag.spv
glslc -O triangle.vert -o triangle.vert.spv
glslc -O cube.frag -o cube.frag.spv
glslc -O cube.vert -o cube.vert.spv
glslc -O cube-texture.frag -o cube-texture.frag.spv
glslc -O cube-texture.vert -o cube-texture.vert.spv