use std::{mem::transmute, path::Path};

use anyhow::{Context, anyhow, bail};
use bon::bon;
use dll_syringe::{
    Syringe,
    process::{BorrowedProcessModule, OwnedProcess},
    rpc::RemotePayloadProcedure,
};
use tracing::{error, info};

use crate::hook::{HookConfig, dll::SET_HOOK};

/// Represents an injected hook with its syringe, payload, and remote set_hook function.
pub struct ShellItemHook {
    syringe: Syringe,
    payload: BorrowedProcessModule<'static>,
    remote_set_hook: RemotePayloadProcedure<fn(Option<HookConfig>)>,
}

impl ShellItemHook {
    /// Enable the hook with the given config.
    pub fn enable_hook(&self, config: HookConfig) {
        let _ = self.remote_set_hook.call(&Some(config));
    }

    /// Disable and detach the hook.
    pub fn disable_hook(&self) {
        let _ = self.remote_set_hook.call(&None);
    }

    /// Eject the DLL from the target process.
    pub fn eject(self) -> Result<(), String> {
        self.syringe.eject(self.payload).map_err(|e| e.to_string())
    }
}

/// A collection of injected hooks that can be ejected together.
pub struct ShellItemHooks {
    hooks: Vec<ShellItemHook>,
}

#[bon]
impl ShellItemHooks {
    /// Inject the hook DLL into all processes with the given name.
    ///
    /// Before [`ShellItemHooks::eject()`], the DLL file will be locked and can't be deleted.
    ///
    /// # Returns
    /// - `Ok(ShellItemHooks)` - Successfully injected hooks
    /// - `Err(anyhow::Error)` - Error during injection
    #[builder]
    pub fn inject(
        /// Path to the hook DLL
        dll_path: &Path,
        /// Name of the process to inject into.
        ///
        /// e.g., "explorer.exe", "dopus.exe", "Totalcmd64.exe"
        #[builder(default = "explorer.exe")]
        process_name: &str,
        /// Path to the log file
        log_path: Option<&Path>,
    ) -> anyhow::Result<Self> {
        if !dll_path.exists() {
            bail!("DLL not found at: {:?}", dll_path);
        }

        // Find all processes with the given name
        // TODO: explorer.exe: File explorer only?
        let processes = OwnedProcess::find_all_by_name(process_name);
        if processes.is_empty() {
            bail!("Failed to find any {} process", process_name);
        }
        info!("Found {} {} processes", processes.len(), process_name);

        // Store injected hooks for later eject
        let mut hooks: Vec<ShellItemHook> = Vec::new();
        for (i, target_process) in processes.into_iter().enumerate() {
            let syringe = Syringe::for_process(target_process);

            info!("[{}] Injecting DLL: {:?}", i, dll_path);
            match syringe.find_or_inject(&dll_path) {
                Ok(payload) => {
                    info!("[{}] Successfully injected hook.dll", i);

                    // Call set_hook to enable the hook
                    let remote_set_hook =
                        unsafe { syringe.get_payload_procedure(payload, SET_HOOK) }
                            .context("Failed to get set_hook procedure")?
                            .context("set_hook not found")?;

                    let config = HookConfig {
                        enabled: true,
                        log: log_path.map(|p| p.to_owned()),
                    };
                    let injected_hook = ShellItemHook {
                        payload: unsafe { transmute(payload) },
                        syringe,
                        remote_set_hook,
                    };
                    injected_hook.enable_hook(config);
                    info!("[{}] Hook enabled with log at {:?}", i, log_path);

                    hooks.push(injected_hook);
                }
                Err(e) => {
                    error!("[{}] Failed to inject DLL: {}", i, e);
                }
            }
        }

        Ok(ShellItemHooks { hooks })
    }

    /// Eject all hooks and return the first error if any.
    pub fn eject(&mut self) -> Result<(), anyhow::Error> {
        let mut first_error: Option<anyhow::Error> = None;

        for hook in self.hooks.drain(..) {
            hook.disable_hook();
            if let Err(e) = hook.eject() {
                error!("Failed to eject DLL: {}", e);
                if first_error.is_none() {
                    first_error = Some(anyhow::anyhow!(e));
                }
            }
        }

        if let Some(e) = first_error {
            return Err(anyhow!(e));
        }

        info!("Successfully ejected DLL");
        Ok(())
    }
}
