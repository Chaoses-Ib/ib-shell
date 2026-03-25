use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use bon::bon;
use ib_hook::{
    inject::dll::app::{DllApp, DllInjectionVecWithInput, OwnedProcess},
    process::{GuiProcessWatcher, Pid},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{app::ShellApp, hook::HookConfig};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub injector: Pid,
    pub config: HookConfig,
}

impl From<HookConfig> for Input {
    fn from(config: HookConfig) -> Self {
        Self {
            injector: Pid::current(),
            config,
        }
    }
}

pub struct ShellDll;
impl DllApp for ShellDll {
    const APPLY: &str = "ib_shell_apply";
    type Input = Input;
    type Output = ();
}

pub struct ShellInjector {
    _watcher: GuiProcessWatcher,
    injections: Arc<Mutex<DllInjectionVecWithInput<ShellDll>>>,
}

#[bon]
impl ShellInjector {
    #[builder]
    pub fn new(
        dll_path: PathBuf,
        config: Option<HookConfig>,
        apps: Vec<ShellApp>,
    ) -> anyhow::Result<Self> {
        let input = config.map(Into::into);
        let injections = DllInjectionVecWithInput::<ShellDll>::with_input(dll_path, input)?;
        let injections = Arc::new(Mutex::new(injections));
        let watcher = ib_hook::process::GuiProcessWatcher::for_each({
            let injections = injections.clone();
            move |pid| {
                let mut injections = injections.lock().unwrap();
                if let Err(e) = injections
                    .inject([OwnedProcess::from_pid(*pid).unwrap()].into_iter())
                    .on_error(|pid, e| error!(%pid, ?e))
                    .call()
                {
                    error!(?e);
                }
            }
        })
        .filter_image_path(move |path| {
            path.and_then(|p| p.file_name())
                .is_some_and(|n| apps.iter().any(|app| n == app.process_name))
        })
        .build()?;
        Ok(Self {
            _watcher: watcher,
            injections,
        })
    }

    pub fn injections<'a>(&'a self) -> MutexGuard<'a, DllInjectionVecWithInput<ShellDll>> {
        self.injections.lock().unwrap()
    }

    pub fn apply(&self, config: HookConfig) {
        self.injections()
            .apply(Input {
                injector: Pid::current(),
                config,
            })
            .on_error(|pid, e| error!(%pid, ?e))
            .call();
    }

    pub fn unapply(&self) {
        self.injections()
            .unapply()
            .on_error(|pid, e| error!(%pid, ?e))
            .call();
    }

    pub fn leak(&self) {
        self.injections().leak();
    }

    pub fn eject(&self) {
        self.injections()
            .eject()
            .on_error(|pid, e| error!(%pid, ?e))
            .call()
    }
}
