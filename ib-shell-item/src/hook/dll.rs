/*!
To export hook DLL APIs:
```
pub use ib_shell_item::hook::dll;
```
*/
use crate::hook;

/*
/// Enable the hook.
#[unsafe(no_mangle)]
pub extern "C" fn ib_shell_item_enable_hook() {
    hook::set_hook(Some(hook::HookConfig {
        enabled: true,
        log: None,
    }));
}

/// Disable the hook.
#[unsafe(no_mangle)]
pub extern "C" fn ib_shell_item_disable_hook() {
    hook::set_hook(None);
}
*/

pub const SET_HOOK: &str = "ib_shell_item_set_hook";

#[dll_syringe::payload_utils::payload_procedure]
fn ib_shell_item_set_hook(config: Option<hook::HookConfig>) {
    hook::set_hook(config)
}
