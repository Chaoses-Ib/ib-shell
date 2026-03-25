use std::{cell::SyncUnsafeCell, ffi::c_void, mem::MaybeUninit, ptr};

use bon::Builder;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use widestring::U16CStr;
use windows::{
    Win32::UI::Shell::SIGDN,
    core::{HRESULT, PWSTR},
};

use crate::{ShellItemDisplayName, hook::HOOK_CONFIG, string};

#[derive(Default, Serialize, Deserialize, Clone, Builder, Debug)]
#[builder(on(Vec<u16>, into))]
pub struct DisplayNameHookConfig {
    /// Mainly for testing.
    display_prefix: Option<Vec<u16>>,
    /// Mainly for testing.
    edit_prefix: Option<Vec<u16>>,
}

pub(crate) type GetDisplayNameFn =
    unsafe extern "system" fn(*mut c_void, SIGDN, *mut PWSTR) -> HRESULT;

/// Store original GetDisplayName function pointer (lazy initialized)
pub(crate) static ORIGINAL_GET_DISPLAY_NAME: SyncUnsafeCell<Option<GetDisplayNameFn>> =
    SyncUnsafeCell::new(None);
pub(crate) static TRUE_GET_DISPLAY_NAME: SyncUnsafeCell<MaybeUninit<GetDisplayNameFn>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

/// Hooked GetDisplayName function
pub(crate) unsafe extern "system" fn sh_get_display_name(
    this: *mut core::ffi::c_void,
    sigdn_name: SIGDN,
    ppsz_name: *mut windows::core::PWSTR,
) -> HRESULT {
    let true_get_display_name = TRUE_GET_DISPLAY_NAME.get();
    let real = || unsafe { (*true_get_display_name).assume_init()(this, sigdn_name, ppsz_name) };

    let config = HOOK_CONFIG.read().unwrap();
    let Some(config) = &config.display_name else {
        return real();
    };

    // Call original function
    let result = real();

    // Log the display name
    if result.is_ok() {
        let name = (unsafe { *ppsz_name }).0;
        match ShellItemDisplayName::try_from(sigdn_name.0) {
            Ok(ShellItemDisplayName::FileSystemPath) => (),
            Ok(d) if d.is_for_display() | d.is_for_edit() => {
                let name = unsafe { U16CStr::from_ptr_str(name) };
                debug!(?d, ?name, "GetDisplayName for display");
                if let Some(prefix) = &config.display_prefix {
                    let new_name = string::prefix_u16cstr_ptr(name, prefix);
                    unsafe { *ppsz_name = new_name };
                }
            }
            Ok(d) if d.is_for_edit() => {
                let name = unsafe { U16CStr::from_ptr_str(name) };
                debug!(?d, ?name, "GetDisplayName for edit");
                if let Some(prefix) = &config.edit_prefix {
                    let new_name = string::prefix_u16cstr_ptr(name, prefix);
                    unsafe { *ppsz_name = new_name };
                }
            }
            Ok(d) => {
                let name = unsafe { U16CStr::from_ptr_str(name) };
                debug!(?d, ?name, "GetDisplayName for parse");
            }
            Err(_) => {
                let name = unsafe { U16CStr::from_ptr_str(name) };
                debug!(?name, "GetDisplayName unknown");
            }
        }
    }

    result
}

fn hook(enable: bool) -> windows::core::Result<()> {
    let res = unsafe {
        slim_detours_sys::SlimDetoursInlineHook(
            enable as _,
            TRUE_GET_DISPLAY_NAME.get().cast(),
            sh_get_display_name as _,
        )
    };
    windows::core::HRESULT(res).ok()
}

pub(crate) fn enable_hook(get_display_name: GetDisplayNameFn) -> windows::core::Result<()> {
    match unsafe { *ORIGINAL_GET_DISPLAY_NAME.get() } {
        Some(f) if ptr::fn_addr_eq(f, get_display_name) => Ok(()),
        None => {
            // Not yet hooked, store original and hook
            unsafe { *ORIGINAL_GET_DISPLAY_NAME.get() = Some(get_display_name) };
            unsafe { (*TRUE_GET_DISPLAY_NAME.get()).write(get_display_name) };
            debug!(?get_display_name, "Hooking GetDisplayName");
            hook(true)
        }
        // Some(f) if ptr::fn_addr_eq(f, sh_get_display_name as GetDisplayNameFn) => Ok(()),
        Some(f) => {
            // TODO
            error!(?f, ?get_display_name, "Multi GetDisplayName");
            windows::core::HRESULT(1).ok()
        }
    }
}

pub(crate) fn disable_hook() -> windows::core::Result<()> {
    if unsafe { *ORIGINAL_GET_DISPLAY_NAME.get() }.is_some() {
        // Unhook and restore original
        debug!("Unhooking GetDisplayName");
        hook(false)
    } else {
        Ok(())
    }
}
