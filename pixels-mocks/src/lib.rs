pub struct RWH;

unsafe impl raw_window_handle::HasRawWindowHandle for RWH {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        #[cfg(target_os = "macos")]
        return raw_window_handle::RawWindowHandle::MacOS(
            raw_window_handle::macos::MacOSHandle::empty(),
        );
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        return raw_window_handle::RawWindowHandle::Wayland(
            raw_window_handle::unix::WaylandHandle::empty(),
        );
        #[cfg(target_os = "windows")]
        return raw_window_handle::RawWindowHandle::Windows(
            raw_window_handle::windows::WindowsHandle::empty(),
        );
        #[cfg(target_os = "ios")]
        return raw_window_handle::RawWindowHandle::IOS(raw_window_handle::ios::IOSHandle::empty());
    }
}
