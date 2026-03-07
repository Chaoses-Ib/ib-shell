use bon::Builder;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Shell::{DefSubclassProc, SetWindowSubclass},
        WindowsAndMessaging::{WM_KILLFOCUS, WM_NCDESTROY, WM_SETFOCUS},
    },
};

use crate::imm::{
    ImeConversionMode, ImeState, get_window_ime_conversion_mode, get_window_ime_state,
    set_ime_conversion_mode, set_window_ime_state,
};

const SUBCLASS_ID: usize = 0;

struct SubclassData {
    config: ImeHookConfig,
    original_state: ImeState,
    original_conversion_mode: ImeConversionMode,
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    #[cfg(test)]
    eprintln!("hwnd: {hwnd:p}, msg: {umsg:X}, wparam: {wparam:X}, wparam: {lparam:X}");

    match umsg {
        WM_SETFOCUS => {
            // Save original IME state
            let original_state = get_window_ime_state(hwnd);
            let original_conversion_mode = get_window_ime_conversion_mode(hwnd);
            #[cfg(feature = "log")]
            {
                eprintln!("get_ime_state: {original_state:?}");
                eprintln!("get_ime_conversion_mode: {original_conversion_mode:?}");
            }

            // Get config from subclass data to determine preferred state
            let subclass_data = unsafe { (data as *mut SubclassData).read() };
            let config = subclass_data.config;

            if let Some(conversion_mode) = config.default_ime_conversion_mode {
                set_ime_conversion_mode(conversion_mode);
            }
            if let Some(state) = config.default_ime_state {
                set_window_ime_state(hwnd, state);
            }

            // Store original state in subclass data
            let original_data = Box::new(SubclassData {
                original_state,
                original_conversion_mode,
                config,
            });
            unsafe { (data as *mut SubclassData).write(*original_data) };
        }
        WM_KILLFOCUS if data != 0 => {
            // Restore original IME state and conversion mode
            let data = unsafe { (data as *mut SubclassData).read() };
            #[cfg(feature = "log")]
            {
                eprintln!("set_ime_state: {:?}", data.original_state);
                eprintln!(
                    "set_ime_conversion_mode: {:?}",
                    data.original_conversion_mode
                );
            }
            set_ime_conversion_mode(data.original_conversion_mode);
            set_window_ime_state(wparam as _, data.original_state);
        }
        WM_NCDESTROY => {
            // Free the subclass data when window is destroyed
            if data != 0 {
                let _ = unsafe { Box::from_raw(data as *mut SubclassData) };
            }
        }
        _ => {}
    }

    let res = unsafe { DefSubclassProc(hwnd, umsg, wparam, lparam) };
    res
}

/// Configuration for IME hook behavior.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Builder)]
pub struct ImeHookConfig {
    pub default_ime_state: Option<ImeState>,
    pub default_ime_conversion_mode: Option<ImeConversionMode>,
}

impl ImeHookConfig {
    pub fn default_off() -> Self {
        Self {
            default_ime_state: Some(false),
            default_ime_conversion_mode: Some(ImeConversionMode::ALPHANUMERIC),
        }
    }
}

impl ImeHookConfig {
    pub fn hook_window(self, hwnd: HWND) -> bool {
        // Allocate subclass data on heap
        let subclass_data = Box::new(SubclassData {
            config: self,
            original_state: false,
            original_conversion_mode: ImeConversionMode::empty(),
        });
        let subclass_data_ptr = Box::into_raw(subclass_data) as usize;

        let result =
            unsafe { SetWindowSubclass(hwnd, Some(wnd_proc), SUBCLASS_ID, subclass_data_ptr) != 0 };

        if !result {
            // Cleanup if setting subclass failed
            let _ = unsafe { Box::from_raw(subclass_data_ptr as *mut SubclassData) };
        }

        result
    }
}
