//! Here be dragons. Abandon all hope, ye who enter.
//!
//! This is probably a bad idea. The purpose of this crate is to move all `unsafe` invocations
//! into a single location and provide a faux safe interface that can be accessed by safe code with
//! `#![forbid(unsafe_code)]`
//!
//! This crate is only intended to be used by `pixels`.

#![deny(clippy::all)]

use raw_window_handle::HasRawWindowHandle;
use wgpu::{Instance, Surface};

/// Create a [`wgpu::Surface`] from the given window handle.
///
/// # Safety
///
/// The window handle must be valid, or very bad things will happen.
pub fn surface_from_window_handle<W: HasRawWindowHandle>(
    instance: &Instance,
    window: &W,
) -> Surface {
    unsafe { instance.create_surface(window) }
}
