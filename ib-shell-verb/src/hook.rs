use std::{cell::SyncUnsafeCell, sync::Mutex};

use tracing::{debug, error};
use widestring::U16CStr;
use windows_sys::{
    Win32::UI::Shell::{SHELLEXECUTEINFOW, ShellExecuteExW},
    core::BOOL,
};

use crate::OpenVerb;

type ShellExecuteExWFn = unsafe extern "system" fn(pexecinfo: *mut SHELLEXECUTEINFOW) -> BOOL;

static TRUE_SHELL_EXECUTE_EX_W: SyncUnsafeCell<ShellExecuteExWFn> =
    SyncUnsafeCell::new(ShellExecuteExW);

/// Hook configuration containing the verbs to try before falling back to default behavior.
#[derive(Default)]
pub struct HookConfig {
    pub verbs: Vec<Box<dyn OpenVerb>>,
}

static HOOK_CONFIG: Mutex<HookConfig> = Mutex::new(HookConfig { verbs: vec![] });

unsafe extern "system" fn shell_execute_ex_w(pexecinfo: *mut SHELLEXECUTEINFOW) -> BOOL {
    let real = || unsafe { (*TRUE_SHELL_EXECUTE_EX_W.get())(pexecinfo) };
    let info = unsafe { &*pexecinfo };

    // Some programs use PIDL to open normal files
    // https://github.com/Chaoses-Ib/IbEverythingExt/issues/104
    let Some(path) =
        ib_shell_item::path::ShellPath::from_path_or_id_list(info.lpFile, info.lpIDList as _)
    else {
        return real();
    };

    // Check if verb is "open"
    let verb = (!info.lpVerb.is_null()).then(|| unsafe { U16CStr::from_ptr_str(info.lpVerb) });
    #[cfg(test)]
    eprintln!("verb: {verb:?}");

    debug!(?verb, ?path);

    if verb.is_none_or(|verb| verb == widestring::u16str!("open")) {
        let config = HOOK_CONFIG.lock().unwrap();

        if let Some(r) = crate::open_verbs(&path, config.verbs.as_slice()) {
            return r.is_ok() as _;
        }
    }

    real()
}

fn hook(enable: bool) -> windows::core::Result<()> {
    let res = unsafe {
        slim_detours_sys::SlimDetoursInlineHook(
            enable as _,
            TRUE_SHELL_EXECUTE_EX_W.get().cast(),
            shell_execute_ex_w as _,
        )
    };
    windows::core::HRESULT(res).ok()
}

/// Set the hook with optional config.
/// If config is None, the hook is disabled.
pub fn set_hook(config: Option<HookConfig>) {
    if let Some(config) = config {
        let mut hook_config = HOOK_CONFIG.lock().unwrap();
        *hook_config = config;
        if let Err(e) = hook(true) {
            error!(%e, "Failed to hook ShellExecuteExW");
        }
    } else {
        if let Err(e) = hook(false) {
            error!(%e, "Failed to detach hook");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        assert_matches::assert_matches,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    struct TestVerb {
        path: Arc<Mutex<Option<PathBuf>>>,
    }

    impl OpenVerb for TestVerb {
        fn handle(&self, path: &Path) -> Option<anyhow::Result<()>> {
            let mut p = self.path.lock().unwrap();
            *p = Some(path.to_path_buf());
            Some(Ok(()))
        }
    }

    #[test]
    fn test_hook_intercepts_open() {
        // Create a mock verb that tracks the path
        let path = Arc::new(Mutex::new(None::<PathBuf>));
        let test_verb = TestVerb { path: path.clone() };

        // Set the hook with config
        set_hook(Some(HookConfig {
            verbs: vec![Box::new(test_verb)],
        }));

        // Use open::that_detached which will call ShellExecuteExW internally
        // The hook should intercept this and call our mock verb
        assert_matches!(open::that_detached("test a"), Ok(_));

        // Verify the mock verb was called with the correct path
        let captured_path = path.lock().unwrap();
        assert_eq!(*captured_path, Some(PathBuf::from("test a")));
    }
}
