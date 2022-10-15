#![deny(clippy::all)]

pub struct Rwh;

unsafe impl raw_window_handle::HasRawWindowHandle for Rwh {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        #[cfg(target_os = "macos")]
        return raw_window_handle::RawWindowHandle::AppKit(
            raw_window_handle::AppKitWindowHandle::empty(),
        );
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        return raw_window_handle::RawWindowHandle::Wayland(
            raw_window_handle::WaylandWindowHandle::empty(),
        );
        #[cfg(target_os = "windows")]
        return raw_window_handle::RawWindowHandle::Win32(
            raw_window_handle::Win32WindowHandle::empty(),
        );
        #[cfg(target_os = "ios")]
        return raw_window_handle::RawWindowHandle::UiKit(
            raw_window_handle::UiKitWindowHandle::empty(),
        );
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for Rwh {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        #[cfg(target_os = "macos")]
        return raw_window_handle::RawDisplayHandle::AppKit(
            raw_window_handle::AppKitDisplayHandle::empty(),
        );
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        return raw_window_handle::RawDisplayHandle::Wayland(
            raw_window_handle::WaylandDisplayHandle::empty(),
        );
        #[cfg(target_os = "windows")]
        return raw_window_handle::RawDisplayHandle::Windows(
            raw_window_handle::WindowsDisplayHandle::empty(),
        );
        #[cfg(target_os = "ios")]
        return raw_window_handle::RawDisplayHandle::UiKit(
            raw_window_handle::UiKitDisplayHandle::empty(),
        );
    }
}
