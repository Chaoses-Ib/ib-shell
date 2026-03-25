use std::{
    fs,
    path::PathBuf,
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use clap::Parser;
use ib_shell::{
    app,
    hook::{HookConfig, inject::ShellInjector},
    item::hook::{
        display_name::DisplayNameHookConfig, inject::ShellItemHooks, property::PropertyHookConfig,
    },
};
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "inject-hook")]
#[command(about = "Inject shell hooks into the process", long_about = None)]
struct Args {
    /// Build profile (debug or release)
    #[arg(short, long)]
    profile: String,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let dll_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target")
        .join(&args.profile)
        .join("examples")
        .join("hook.dll");

    let log_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("hook.log");
    dbg!(&log_path);
    _ = fs::remove_file(&log_path);

    let config = HookConfig::builder()
        .item(
            ib_shell::item::hook::HookConfig::builder()
                .enabled(true)
                .display_name(
                    DisplayNameHookConfig::builder()
                        .display_prefix(widestring::u16str!("😭").as_slice())
                        .edit_prefix(widestring::u16str!("😭").as_slice())
                        .build(),
                )
                .property({
                    let property = PropertyHookConfig::builder()
                        .str_prefix(widestring::u16str!("💢").as_slice());
                    #[cfg(feature = "everything")]
                    let property = property.size_from_everything(true);
                    property.build()
                })
                .log(log_path)
                .build(),
        )
        .build();
    let mut hooks = match ShellInjector::builder()
        .dll_path(dll_path)
        .config(config)
        // .apps(vec![app::NOTEPAD])
        .apps(vec![app::EXPLORER])
        .build()
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
    hooks.eject();
}
