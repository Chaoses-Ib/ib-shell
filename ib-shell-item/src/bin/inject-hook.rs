use std::{
    path::PathBuf,
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use ib_shell_item::hook::inject::ShellItemHooks;
use tracing::{error, info};

fn main() {
    tracing_subscriber::fmt::init();

    let dll_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("examples")
        .join("hook.dll");

    let log_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("hook.log");

    let mut hooks = match ShellItemHooks::inject()
        .dll_path(&dll_path)
        .log_path(&log_path)
        .call()
    {
        Ok(hooks) => hooks,
        Err(e) => {
            error!("Error: {}", e);
            exit(1);
        }
    };

    info!("Press Ctrl+C to eject and exit...");

    // Set up signal handler for Ctrl+C
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Wait for Ctrl+C
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(1));
    }

    info!("Disabling hooks and ejecting DLL...");

    if let Err(e) = hooks.eject() {
        error!("Error ejecting: {}", e);
        exit(1);
    }

    info!("Successfully ejected DLL");
}
