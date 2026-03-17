use std::{cell::SyncUnsafeCell, ffi::c_void, mem::MaybeUninit, ptr};

use bon::Builder;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use windows::{
    Win32::UI::Shell::{
        IShellItem2,
        PropertiesSystem::{GETPROPERTYSTOREFLAGS, IPropertyStore},
    },
    core::{GUID, HRESULT, Interface},
};

use crate::hook::HOOK_CONFIG;

pub mod value;

#[derive(Default, Serialize, Deserialize, Clone, Builder)]
#[builder(on(Vec<u16>, into))]
pub struct PropertyHookConfig {
    /// Mainly for testing.
    str_prefix: Option<Vec<u16>>,
}

pub(crate) type GetPropertyStoreFn = unsafe extern "system" fn(
    *mut c_void,
    GETPROPERTYSTOREFLAGS,
    *const GUID,
    *mut *mut c_void,
) -> HRESULT;

/// Store original and real GetPropertyStore function pointers (lazy initialized)
pub(crate) struct GetPropertyStoreState {
    pub original_get_store: Option<GetPropertyStoreFn>,
    pub real_get_store: MaybeUninit<GetPropertyStoreFn>,
}

pub(crate) static GET_PROPERTY_STORE_STATE: SyncUnsafeCell<GetPropertyStoreState> =
    SyncUnsafeCell::new(GetPropertyStoreState {
        original_get_store: None,
        real_get_store: MaybeUninit::uninit(),
    });

/// Hooked GetPropertyStore function
///
/// [IShellItem2::GetPropertyStore (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitem2-getpropertystore)
pub(crate) unsafe extern "system" fn get_property_store(
    this: *mut c_void,
    flags: GETPROPERTYSTOREFLAGS,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let state = GET_PROPERTY_STORE_STATE.get();
    let real = || unsafe { (*state).real_get_store.assume_init()(this, flags, riid, ppv) };

    let config = HOOK_CONFIG.read().unwrap();
    let Some(_config) = &config.property else {
        return real();
    };

    // Call original function
    let result = real();

    if result.is_ok() {
        debug!(?flags, "GetPropertyStore called");
        let store = unsafe { IPropertyStore::from_raw_borrowed(&*ppv) }.unwrap();
        if let Err(e) = value::enable_hook(store) {
            error!(%e, "Failed to hook GetValue");
        }
    }

    result
}

fn hook(enable: bool) -> windows::core::Result<()> {
    let state = GET_PROPERTY_STORE_STATE.get();
    let res = unsafe {
        slim_detours_sys::SlimDetoursInlineHook(
            enable as _,
            (*state).real_get_store.as_mut_ptr().cast(),
            get_property_store as _,
        )
    };
    windows::core::HRESULT(res).ok()
}

pub(crate) fn enable_hook(item2: &IShellItem2) -> windows::core::Result<()> {
    let get_property_store = item2.vtable().GetPropertyStore;

    // Check if already initialized with the same function
    let state = unsafe { &mut *GET_PROPERTY_STORE_STATE.get() };
    match state.original_get_store {
        Some(f) if ptr::fn_addr_eq(f, get_property_store) => Ok(()),
        None => {
            // Not yet initialized, write the state
            state.original_get_store = Some(get_property_store);
            (*state).real_get_store.write(get_property_store);
            debug!(?get_property_store, "Hooking GetPropertyStore");
            hook(true)
        }
        Some(f) => {
            // TODO
            error!(?f, ?get_property_store, "Multi GetPropertyStore");
            windows::core::HRESULT(1).ok()
        }
    }
}

pub(crate) fn disable_hook() -> windows::core::Result<()> {
    let state = GET_PROPERTY_STORE_STATE.get();
    if unsafe { (*state).original_get_store.is_some() } {
        // Unhook and restore original
        debug!("Unhooking GetPropertyStore");
        hook(false)?;
    }
    value::disable_hook()
}
