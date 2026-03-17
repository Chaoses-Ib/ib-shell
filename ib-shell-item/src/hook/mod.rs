/*!
## Remote processes
There are mainly three ways to hook remote processes:
- [`inject`]: Inject a [`DLL`](dll) directly
  - Controllable target processes.
  - Easily hot reload.
  - Hard to manage for multi-process applications (Explorer).
  - May cause antivirus false positives.
- Register a Shell extension
  - Require system (Registry) changes.
  - Hard to hot reload, since the extension will be loaded into many processes.
- DLL hijacking
  - Only suitable for third-party applications.

## Applications
- As a performance/shittiness measure.
  - Windows 11 24H2 Explorer: 2000 calls/folder
  - DOpus: 250 calls/folder
  - TC: 0?
*/
use std::{cell::SyncUnsafeCell, path::PathBuf, sync::RwLock};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace};
use windows::Win32::UI::Shell::IShellItem;
use windows_sys::{
    Win32::UI::Shell::Common::ITEMIDLIST,
    core::{GUID, HRESULT},
};

use crate::{ShellItem, ShellItemDisplayName};

#[cfg(feature = "hook-dll")]
pub mod dll;
#[cfg(feature = "hook-dll")]
pub mod inject;

type SHCreateItemFromIDListFn = unsafe extern "system" fn(
    pidl: *const ITEMIDLIST,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT;

// shell32.dll!SHCreateItemFromIDList is actually implemented in windows.storage.dll
windows_link::link!("windows.storage.dll" "system" "SHCreateItemFromIDList" fn SHCreateItemFromIDList_windows_storage(pidl : *const ITEMIDLIST, riid : *const GUID, ppv : *mut *mut core::ffi::c_void) -> HRESULT);

static TRUE_SH_CREATE_ITEM_FROM_ID_LIST: SyncUnsafeCell<SHCreateItemFromIDListFn> =
    SyncUnsafeCell::new(SHCreateItemFromIDList_windows_storage);

/// Hook configuration for [`SHCreateItemFromIDList`].
/// This is used to intercept shell item creation from ID lists.
#[derive(Default, Serialize, Deserialize)]
pub struct HookConfig {
    /// If true, the hook will intercept all [`SHCreateItemFromIDList]` calls.
    pub enabled: bool,
    /// Existing logs in the log file won't be cleared.
    ///
    /// Ignored if `hook-log` feature is not enabled.
    pub log: Option<PathBuf>,
}

static HOOK_CONFIG: RwLock<HookConfig> = RwLock::new(HookConfig {
    enabled: false,
    log: None,
});

unsafe extern "system" fn sh_create_item_from_id_list(
    pidl: *const ITEMIDLIST,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let real = || unsafe { (*TRUE_SH_CREATE_ITEM_FROM_ID_LIST.get())(pidl, riid, ppv) };

    let result = real();

    let config = HOOK_CONFIG.read().unwrap();
    if !config.enabled {
        return result;
    }

    // If successful, get and log the display name
    trace!(?pidl, ?riid, ?ppv, ?result, "SHCreateItemFromIDList called");
    if result >= 0 {
        let ppv = ppv as *mut IShellItem;
        let item = unsafe { &*ppv };
        let name = ShellItem::get_display_name(item, ShellItemDisplayName::FileSystemPath);
        debug!(?name, "SHCreateItemFromIDList called");
    } else {
        debug!(?result, "SHCreateItemFromIDList called");
    }

    result
}

fn hook(enable: bool) -> windows::core::Result<()> {
    let res = unsafe {
        slim_detours_sys::SlimDetoursInlineHook(
            enable as _,
            TRUE_SH_CREATE_ITEM_FROM_ID_LIST.get().cast(),
            sh_create_item_from_id_list as _,
        )
    };
    windows::core::HRESULT(res).ok()
}

/// Initialize logging if log path is set in config.
#[cfg(feature = "hook-log")]
fn log_init(log_path: &PathBuf) {
    // Syncly log in debug mode
    #[cfg(debug_assertions)]
    let writer = {
        let log_dir = log_path.parent().unwrap();
        let log_filename = log_path.file_name().unwrap();
        tracing_appender::rolling::never(log_dir, log_filename)
    };
    #[cfg(not(debug_assertions))]
    let (writer, _guard) = tracing_appender::non_blocking(
        std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_path)
            .ok()
            .unwrap(),
    );

    let _ = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .try_init();
    info!("log_init");
}

/// Set the hook with optional config.
/// If config is None or enabled is false, the hook is disabled.
pub fn set_hook(config: Option<HookConfig>) {
    if let Some(config) = config {
        let mut hook_config = HOOK_CONFIG.write().unwrap();
        *hook_config = config;
        if hook_config.enabled {
            #[cfg(feature = "hook-log")]
            if let Some(ref log_path) = hook_config.log {
                log_init(log_path);
            }
            info!("attach");
            if let Err(e) = hook(true) {
                error!(%e, "Failed to hook SHCreateItemFromIDList");
            }
        }
    } else {
        info!("detach");
        if let Err(e) = hook(false) {
            error!(%e, "Failed to detach hook");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_config_default() {
        let config = HookConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn set_hook_none() {
        // Should not panic
        set_hook(None);
    }

    #[test]
    fn set_hook_disabled() {
        set_hook(Some(HookConfig {
            enabled: false,
            ..Default::default()
        }));
    }
}
