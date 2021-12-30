#![deny(clippy::all)]

pub struct Rwh;

unsafe impl raw_window_handle::HasRawWindowHandle for Rwh {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        #[cfg(target_os = "macos")]
        return raw_window_handle::RawWindowHandle::AppKit(raw_window_handle::AppKitHandle::empty());
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        return raw_window_handle::RawWindowHandle::Wayland(
            raw_window_handle::WaylandHandle::empty(),
        );
        #[cfg(target_os = "windows")]
        return raw_window_handle::RawWindowHandle::Win32(raw_window_handle::Win32Handle::empty());
        #[cfg(target_os = "ios")]
        return raw_window_handle::RawWindowHandle::UiKit(raw_window_handle::UiKitHandle::empty());
    }
}
