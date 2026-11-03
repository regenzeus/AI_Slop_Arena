//! Windows input that works while the game is in the background (UI-20).
//!
//! Bevy reads controllers through Windows.Gaming.Input, which only reports
//! input while the game window has focus. XInput reports it always, so the
//! game reads Xbox controllers itself. Print Screen is read from the key's
//! physical state, which a screenshot tool that grabs the key cannot hide.
//! On other systems these report nothing and Bevy's input is used.

/// One XInput controller's state: buttons, triggers and sticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pad {
    pub buttons: u16,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub lx: i16,
    pub ly: i16,
}

impl Pad {
    pub const DPAD_UP: u16 = 0x0001;
    pub const DPAD_DOWN: u16 = 0x0002;
    pub const DPAD_LEFT: u16 = 0x0004;
    pub const DPAD_RIGHT: u16 = 0x0008;
    pub const START: u16 = 0x0010;
    pub const LB: u16 = 0x0100;
    pub const RB: u16 = 0x0200;
    pub const A: u16 = 0x1000;
    pub const B: u16 = 0x2000;
    pub const X: u16 = 0x4000;
    pub const Y: u16 = 0x8000;
}

#[cfg(windows)]
mod sys {
    use super::Pad;
    use std::sync::OnceLock;

    #[repr(C)]
    #[derive(Default)]
    struct XInputState {
        packet: u32,
        buttons: u16,
        left_trigger: u8,
        right_trigger: u8,
        lx: i16,
        ly: i16,
        rx: i16,
        ry: i16,
    }

    type GetState = unsafe extern "system" fn(u32, *mut XInputState) -> u32;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryA(name: *const core::ffi::c_char) -> *mut core::ffi::c_void;
        fn GetProcAddress(module: *mut core::ffi::c_void, name: *const core::ffi::c_char) -> *mut core::ffi::c_void;
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetAsyncKeyState(key: i32) -> i16;
    }

    /// XInputGetState from whichever XInput DLL the system has.
    fn get_state() -> Option<GetState> {
        static FN: OnceLock<Option<usize>> = OnceLock::new();
        let f = FN.get_or_init(|| {
            for dll in [c"xinput1_4.dll", c"xinput1_3.dll", c"xinput9_1_0.dll"] {
                // SAFETY: the names are NUL-terminated; a null module is checked.
                let module = unsafe { LoadLibraryA(dll.as_ptr()) };
                if module.is_null() {
                    continue;
                }
                // SAFETY: as above.
                let proc = unsafe { GetProcAddress(module, c"XInputGetState".as_ptr()) };
                if !proc.is_null() {
                    return Some(proc as usize);
                }
            }
            None
        });
        // SAFETY: the address is XInputGetState, which has this signature.
        f.map(|p| unsafe { std::mem::transmute::<usize, GetState>(p) })
    }

    pub fn pads() -> Vec<Pad> {
        let Some(get) = get_state() else { return Vec::new() };
        let mut out = Vec::new();
        for user in 0..4 {
            let mut s = XInputState::default();
            // SAFETY: `s` is a valid XINPUT_STATE to write into.
            if unsafe { get(user, &mut s) } == 0 {
                out.push(Pad { buttons: s.buttons, left_trigger: s.left_trigger, right_trigger: s.right_trigger, lx: s.lx, ly: s.ly });
            }
        }
        out
    }

    /// Held now, and pressed since the last call.
    pub fn print_screen() -> (bool, bool) {
        // SAFETY: a plain query of a virtual key's state (VK_SNAPSHOT).
        let s = unsafe { GetAsyncKeyState(0x2C) } as u16;
        (s & 0x8000 != 0, s & 0x0001 != 0)
    }
}

#[cfg(not(windows))]
mod sys {
    pub fn pads() -> Vec<super::Pad> {
        Vec::new()
    }

    pub fn print_screen() -> (bool, bool) {
        (false, false)
    }
}

/// Every connected XInput controller.
pub fn pads() -> Vec<Pad> {
    sys::pads()
}

/// Whether Print Screen is held now, and whether it was pressed since the
/// last call.
pub fn print_screen() -> (bool, bool) {
    sys::print_screen()
}
