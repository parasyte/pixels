//! Provides a macro and type for including SPIR-V shaders in const data.
//!
//! In an ideal world, a shader will be compiled at build-time directly into the executable. This
//! is opposed to the typical method of including a shader, which reads a GLSL source code file
//! from the file system at start, compiles it, and sends it to the GPU. That process adds a
//! non-trivial amount of time to startup, and additional error handling code at runtime.
//!
//! This macro moves all of that complexity to build-time. At least for the SPIR-V part of the
//! shader pipeline. (`gfx-hal` backends have their own SPIR-V-to-native compilers at runtime.)
//!
//! Cribbed with permission from Ralith
//! See: https://github.com/MaikKlein/ash/pull/245

/// Include correctly aligned and typed precompiled SPIR-V
///
/// Does not account for endianness mismatches between the SPIR-V file and the target. See
/// [`wgpu::read_spirv`] for a more general solution.
#[macro_export]
macro_rules! include_spv {
    ($path:expr) => {
        &$crate::Align4(*include_bytes!($path)) as &$crate::Spirv
    };
}

/// Type returned by `include_spv`, convertible to `&[u32]`
///
/// The definition of this type is unstable.
pub type Spirv = Align4<[u8]>;

impl std::ops::Deref for Spirv {
    type Target = Vec<u32>;
    fn deref(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(self.0.len() / 4);
        for i in 0..self.0.len() / 4 {
            let mut tmp: u32 = 0;
            tmp += (self.0[i] << 24) as u32;
            tmp += (self.0[i + 1] << 16) as u32;
            tmp += (self.0[i + 2] << 8) as u32;
            tmp += self.0[i + 3] as u32;
            out.push(tmp);
        }
        out
    }
}

#[repr(align(4))]
#[doc(hidden)]
pub struct Align4<T: ?Sized>(pub T);
