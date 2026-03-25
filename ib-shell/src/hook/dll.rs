use ib_hook::inject::dll::app::DllApp;

use crate::hook::{inject::ShellDll, set_hook};

ib_hook::inject::dll::app::export_apply!(apply_hook, "ib_shell_apply");

fn apply_hook(input: Option<<ShellDll as DllApp>::Input>) -> <ShellDll as DllApp>::Output {
    if let Some(input) = &input {
        ib_hook::inject::dll::dll::spawn_wait_and_free_current_module_once!(input.injector, || {
            apply_hook(None);
            0
        });
    }
    set_hook(input.map(|i| i.config));
}
