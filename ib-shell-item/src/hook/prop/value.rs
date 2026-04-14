#![allow(non_upper_case_globals)]
use std::{
    cell::{LazyCell, SyncUnsafeCell},
    ffi::c_void,
    mem::MaybeUninit,
    ptr,
};

use tracing::{debug, error, trace};
#[cfg(feature = "everything")]
use windows::Win32::Storage::EnhancedStorage::PKEY_Size;
use windows::{
    Win32::{
        Foundation::PROPERTYKEY,
        Storage::EnhancedStorage::PKEY_ParsingPath,
        System::{
            Com::StructuredStorage::PROPVARIANT,
            Variant::{VT_BSTR, VT_LPWSTR},
        },
        UI::Shell::PropertiesSystem::IPropertyStore,
    },
    core::{BSTR, HRESULT, Interface},
};

use crate::{hook::HOOK_CONFIG, prop::store::PropertyStore};

pub(crate) type GetValueFn =
    unsafe extern "system" fn(*mut c_void, *const PROPERTYKEY, *mut PROPVARIANT) -> HRESULT;

/// Store original and real GetValue function pointers (lazy initialized)
pub(crate) struct GetValueState {
    pub original_get_value: Option<GetValueFn>,
    pub real_get_value: MaybeUninit<GetValueFn>,
}

pub(crate) static GET_VALUE_STATE: SyncUnsafeCell<GetValueState> =
    SyncUnsafeCell::new(GetValueState {
        original_get_value: None,
        real_get_value: MaybeUninit::uninit(),
    });

/// Hooked GetValue function
///
/// [IPropertyStore::GetValue (propsys.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-ipropertystore-getvalue)
///
/// [PROPVARIANT (propidlbase.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/propidlbase/ns-propidlbase-propvariant)
pub(crate) unsafe extern "system" fn get_value(
    this: *mut c_void,
    pkey: *const PROPERTYKEY,
    pv: *mut PROPVARIANT,
) -> HRESULT {
    let state = GET_VALUE_STATE.get();
    let real = || unsafe { (*state).real_get_value.assume_init()(this, pkey, pv) };

    let config = HOOK_CONFIG.read().unwrap();
    let Some(config) = &config.property else {
        return real();
    };

    // Call original function
    let result = real();

    if result.is_ok() {
        let store = unsafe { IPropertyStore::from_raw_borrowed(&this) }.unwrap();
        let pkey = unsafe { &*pkey };
        let v = unsafe { &*pv };
        let path = LazyCell::new(|| store.get_parsing_path().unwrap_or_default());
        match *pkey {
            PKEY_ParsingPath => return result,
            #[cfg(feature = "everything")]
            PKEY_Size => {
                if config.size_from_everything && v.is_empty() {
                    /*
                    #[cfg(debug_assertions)]
                    let t = std::time::Instant::now();
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_time()
                        .build()
                        .unwrap();
                    */
                    match {
                        /*
                        let r = rt.block_on(async {
                            #[cfg(debug_assertions)]
                            debug!(t1 = ?t.elapsed());
                            everything_ipc::pipe::EverythingClient::builder()
                                .build()
                                .await
                                .and_then(|client| {
                                    // ~20us
                                    #[cfg(debug_assertions)]
                                    debug!(t2 = ?t.elapsed());
                                    // TODO: Preread folder
                                    client.get_folder_size_from_filename(&path.to_string())
                                })
                        });
                        // 100~200us
                        #[cfg(debug_assertions)]
                        debug!(t3 = ?t.elapsed());
                        r
                        */
                        use std::path::Path;

                        let mut max = 0;
                        everything_ipc::folder::size::get_folder_size(Path::new(&path.to_string()))
                            .parent_max_size(&mut max)
                            .eager_get_links(true)
                            .call()
                            .map(|size| (size, max))
                    } {
                        Ok((size, max)) => {
                            use crate::hook::prop::magic;

                            debug!(path = %*path, size, "everything");
                            unsafe { *pv = size.into() };
                            magic::ui8_write_u64(pv, max);
                            return result;
                        }
                        Err(e) => error!(?e, path = %*path, "everything"),
                    }
                }
            }
            _ => trace!(path = %*path, ?pkey, ?v, "GetValue"),
        }
        match v.vt() {
            VT_BSTR | VT_LPWSTR => {
                if let Some(prefix) = &config.str_prefix {
                    // VT_LPWSTR: or v.Anonymous.Anonymous.Anonymous.pwszVal, but WTF...
                    if let Ok(str) = BSTR::try_from(v) {
                        let new_str = prefix
                            .iter()
                            .cloned()
                            .chain(str.iter().cloned())
                            .collect::<Vec<_>>();
                        unsafe { *pv = BSTR::from_wide(&new_str).into() };
                    }
                }
            }
            _ => (),
        }
    }

    result
}

fn hook(enable: bool) -> windows::core::Result<()> {
    let state = GET_VALUE_STATE.get();
    let res = unsafe {
        slim_detours_sys::SlimDetoursInlineHook(
            enable as _,
            (*state).real_get_value.as_mut_ptr().cast(),
            get_value as _,
        )
    };
    windows::core::HRESULT(res).ok()
}

pub(crate) fn enable_hook(
    store: &windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore,
) -> windows::core::Result<()> {
    let get_value = store.vtable().GetValue;
    let state = unsafe { &mut *GET_VALUE_STATE.get() };

    match state.original_get_value {
        Some(f) if ptr::fn_addr_eq(f, get_value) => Ok(()),
        None => {
            // Not yet initialized, write the state
            state.original_get_value = Some(get_value);
            (*state).real_get_value.write(get_value);
            debug!(?get_value, "Hooking GetValue");
            hook(true)
        }
        Some(f) => {
            // TODO
            error!(?f, ?get_value, "Multi GetValue");
            windows::core::HRESULT(1).ok()
        }
    }
}

pub(crate) fn disable_hook() -> windows::core::Result<()> {
    let state = GET_VALUE_STATE.get();
    if unsafe { (*state).original_get_value.is_some() } {
        debug!("Unhooking GetValue");
        hook(false)
    } else {
        Ok(())
    }
}
