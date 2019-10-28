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
    type Target = [u32];
    fn deref(&self) -> &[u32] {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            std::slice::from_raw_parts(self.0.as_ptr() as *const u32, self.0.len() / 4)
        }
    }
}

#[repr(align(4))]
#[doc(hidden)]
pub struct Align4<T: ?Sized>(pub T);
