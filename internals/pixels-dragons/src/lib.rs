//! Here be dragons. Abandon all hope, ye who enter.

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
