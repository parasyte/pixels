//! This module implements the bare minimum of a high-level abstraction for SDL2 over fermium.
//!
//! We used to use beryllium, but that crate has lagged behind fermium in maintenance releases.

use ::pixels::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use fermium::prelude::*;
use thiserror::Error;
use zstring::ZStr;

pub struct Sdl {
    sdl_win: *mut SDL_Window,
    _winfo: SDL_SysWMinfo,
    rwh: RawWindowHandle,
}

#[derive(Debug, Error)]
pub enum SdlError {
    #[error("SDL init failed")]
    Init,

    #[error("Invalid pixel buffer size")]
    Size,

    #[error("Create SDL window failed")]
    Window,

    #[error("Create SDL WM Info failed")]
    WMinfo,

    #[error("Create Window Handle failed")]
    WindowHandle,
}

#[derive(Debug)]
pub enum Event {
    Quit,
    Keyboard(SDL_Keycode),
    WindowResized { width: u32, height: u32 },
}

impl Sdl {
    pub fn new(title: ZStr, width: u32, height: u32) -> Result<Self, SdlError> {
        // SAFETY: Ensure SDL is initialized and we get a non-null Window pointer.
        unsafe {
            if SDL_Init(SDL_INIT_EVERYTHING) != 0 {
                return Err(SdlError::Init);
            }

            let window_type = if cfg!(target_os = "macos") {
                SDL_WINDOW_METAL
            } else {
                SDL_WINDOW_VULKAN
            };

            let sdl_win = SDL_CreateWindow(
                title.as_ptr() as *const i8,
                SDL_WINDOWPOS_CENTERED,
                SDL_WINDOWPOS_CENTERED,
                width.try_into().map_err(|_| SdlError::Size)?,
                height.try_into().map_err(|_| SdlError::Size)?,
                (window_type | SDL_WINDOW_ALLOW_HIGHDPI | SDL_WINDOW_RESIZABLE).0,
            );
            if sdl_win.is_null() {
                return Err(SdlError::Window);
            }

            let mut winfo = SDL_SysWMinfo::default();
            if SDL_GetWindowWMInfo(sdl_win, &mut winfo) == SDL_FALSE {
                return Err(SdlError::WMinfo);
            }

            let rwh = winfo.try_into().ok_or(SdlError::WindowHandle)?;

            Ok(Self {
                sdl_win,
                _winfo: winfo,
                rwh,
            })
        }
    }

    // XXX: Fermium doesn't support `SDL_Metal_GetDrawableSize`
    #[cfg(not(target_os = "macos"))]
    pub fn drawable_size(&self) -> (u32, u32) {
        // SAFETY: We have a valid Vulkan window.
        unsafe {
            let mut width = 0;
            let mut height = 0;
            SDL_Vulkan_GetDrawableSize(self.sdl_win, &mut width, &mut height);

            (width.try_into().unwrap(), height.try_into().unwrap())
        }
    }

    #[must_use]
    pub fn poll_event(&self) -> Option<Event> {
        let mut e = SDL_Event::default();
        if unsafe { SDL_PollEvent(&mut e) } == 0 {
            None
        } else {
            Event::try_from(e).ok()
        }
    }
}

unsafe impl HasRawWindowHandle for Sdl {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.rwh
    }
}

impl TryFrom<SDL_Event> for Event {
    type Error = ();

    fn try_from(event: SDL_Event) -> Result<Self, Self::Error> {
        // SAFETY: Only access union fields in when the combinations are known to be valid.
        unsafe {
            Ok(match event.type_ {
                SDL_QUIT => Self::Quit,
                SDL_KEYDOWN | SDL_KEYUP => Self::Keyboard(event.key.keysym.sym),
                SDL_WINDOWEVENT => match event.window.event {
                    SDL_WINDOWEVENT_RESIZED => Self::WindowResized {
                        width: event.window.data1 as _,
                        height: event.window.data2 as _,
                    },
                    _ => return Err(()),
                },
                _ => return Err(()),
            })
        }
    }
}
