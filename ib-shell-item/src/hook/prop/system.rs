#![allow(non_upper_case_globals)]
use std::{cell::SyncUnsafeCell, ffi::c_void};

use bon::Builder;
use ib_hook::inline::InlineHook;
use serde::{Deserialize, Serialize};
use tracing::trace;
use widestring::U16CStr;
use windows::{
    Win32::{
        Foundation::{PROPERTYKEY, S_OK},
        Storage::EnhancedStorage::PKEY_Size,
        System::Com::StructuredStorage::PROPVARIANT,
        UI::Shell::PropertiesSystem::{IPropertySystem, PDFF_ALWAYSKB, PROPDESC_FORMAT_FLAGS},
    },
    core::{HRESULT, Interface, PWSTR},
};

use crate::{hook::prop::magic, prop::system::PropertySystem, string::prefix_u16cstr_ptr};

#[derive(Default, Serialize, Deserialize, Clone, Builder, Debug)]
pub struct PropertySystemHookConfig {
    #[builder(default)]
    size_no_alwayskb: bool,
    #[builder(default)]
    size_max_bar: bool,
}

type FormatForDisplayAlloc = unsafe extern "system" fn(
    *mut c_void,
    *const PROPERTYKEY,
    *const PROPVARIANT,
    PROPDESC_FORMAT_FLAGS,
    *mut PWSTR,
) -> HRESULT;

pub(crate) struct PropertySystemHook {
    format_for_display_alloc: Option<InlineHook<FormatForDisplayAlloc>>,
    config: PropertySystemHookConfig,
}

impl PropertySystemHook {
    pub fn new(config: PropertySystemHookConfig) -> anyhow::Result<Self> {
        let system = IPropertySystem::new()?;

        let format_for_display_alloc = if config.size_max_bar || config.size_no_alwayskb {
            let format_for_display_alloc = system.vtable().FormatForDisplayAlloc;
            InlineHook::new_enabled(format_for_display_alloc, format_for_display_alloc_detour).ok()
        } else {
            None
        };

        Ok(Self {
            format_for_display_alloc,
            config,
        })
    }
}

pub(crate) static HOOK: SyncUnsafeCell<Option<PropertySystemHook>> = SyncUnsafeCell::new(None);

pub fn apply(config: Option<PropertySystemHookConfig>) -> anyhow::Result<()> {
    let hook = unsafe { &mut *HOOK.get() };
    *hook = match config {
        Some(config) => Some(PropertySystemHook::new(config)?),
        None => None,
    };
    Ok(())
}

#[allow(non_snake_case)]
unsafe extern "system" fn format_for_display_alloc_detour(
    this: *mut c_void,
    key: *const PROPERTYKEY,
    propvar: *const PROPVARIANT,
    pdff: PROPDESC_FORMAT_FLAGS,
    ppszDisplay: *mut PWSTR,
) -> HRESULT {
    trace!(?key, ?propvar, ?pdff);
    let hook = unsafe { &*HOOK.get() }.as_ref().unwrap();
    let trampoline = hook.format_for_display_alloc.as_ref().unwrap().trampoline();
    let real = || unsafe { trampoline(this, key, propvar, pdff, ppszDisplay) };

    // let system = unsafe { IPropertySystem::from_raw_borrowed(&this) }.unwrap();
    let pkey = unsafe { &*key };
    match *pkey {
        PKEY_Size => {
            if hook.config.size_no_alwayskb || hook.config.size_max_bar {
                let r = unsafe {
                    trampoline(
                        this,
                        key,
                        propvar,
                        if hook.config.size_no_alwayskb {
                            pdff & !PDFF_ALWAYSKB
                        } else {
                            pdff
                        },
                        ppszDisplay,
                    )
                };
                if r.is_err() {
                    return r;
                }
            }
            if hook.config.size_max_bar
                && let Some(size) = TryInto::<u64>::try_into(unsafe { &*propvar }).ok()
                && let Some(max) = magic::ui8_read_u64(propvar)
            {
                let s = unsafe { U16CStr::from_ptr_str((*ppszDisplay).0) };

                // If too wide, the size column will be truncated from unit to numbers...
                let bar_width = 25;
                let n = if max == 0 {
                    1
                } else {
                    std::cmp::min((size * bar_width / max) + 1, bar_width)
                };
                // Fortunately, the default font is not monospaced so making bar is easy.
                // 1px per char.
                // Unfortunately, we also can't overlap bar with size string easily.
                // let bar = "\u{258F}".repeat(n as usize).to_string() + " ";
                let mut bar = widestring::utf16str!("\u{258F}").repeat(n as usize);
                bar.push(' ');

                // let s = format!("{bar} {:.0} KB", size / 1024);
                // unsafe { *ppszDisplay = str_to_co_task(&s) };
                // let prefix: Vec<u16> = bar.encode_utf16().collect();
                let prefix = bar.as_slice();
                unsafe { *ppszDisplay = prefix_u16cstr_ptr(s, &prefix) };
            };
            return S_OK;
        }
        _ => (),
    }

    real()
}
