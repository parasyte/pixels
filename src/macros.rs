/// Provides a macro and type for including SPIR-V shaders in const data.
///
/// In an ideal world, a shader will be compiled at build-time directly into the executable. This
/// is opposed to the typical method of including a shader, which reads a GLSL source code file
/// from the file system at start, compiles it, and sends it to the GPU. That process adds a
/// non-trivial amount of time to startup, and additional error handling code at runtime.
///
/// This macro moves all of that complexity to build-time. At least for the SPIR-V part of the
/// shader pipeline. (`gfx-hal` backends have their own SPIR-V-to-native compilers at runtime.)

#[macro_export]
macro_rules! include_spv {
    ($path:expr) => {
        &wgpu::read_spirv(std::io::Cursor::new(&include_bytes!($path)[..]))
            .expect(&format!("Invalid SPIR-V shader in file: {}", $path))
    };
}
