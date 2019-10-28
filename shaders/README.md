# Shaders

The GLSL shader source is not compiled as part of the normal cargo build process. This was a conscious decision sparked by the current state of the ecosystem; compiling GLSL-to-SPIR-V requires a C++ toolchain including CMake, which is an unacceptable constraint for a simple crate providing a pixel buffer.

If you need to modify the GLSL sources, you must also recompile the SPIR-V as well. This can be done with `glslang`, `glslc`, etc.

Compile shaders with `glslangValidator`:

```bash
glslangValidator -V shader.frag && glslangValidator -V shader.vert
```

For more information, see https://github.com/parasyte/pixels/issues/9
