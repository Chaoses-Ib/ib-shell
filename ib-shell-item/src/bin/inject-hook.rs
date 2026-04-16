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
use ib_shell_item::hook::{
    HookConfig,
    display_name::DisplayNameHookConfig,
    folder::FolderHookConfig,
    inject::ShellItemHooks,
    prop::{PropertyHookConfig, system::PropertySystemHookConfig},
};
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "inject-hook")]
#[command(about = "Inject shell item hooks into the process", long_about = None)]
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
    _ = fs::remove_file(&log_path);

    let mut hooks = match ShellItemHooks::inject()
        .dll_path(&dll_path)
        .config(
            HookConfig::builder()
                .enabled(true)
                .display_name(
                    DisplayNameHookConfig::builder()
                        .display_prefix(widestring::u16str!("😭").as_slice())
                        .edit_prefix(widestring::u16str!("😭").as_slice())
                        .build(),
                )
                .folder({
                    let folder = FolderHookConfig::builder();
                    #[cfg(feature = "everything")]
                    let folder = folder.compare_size_from_everything(true);
                    folder.build()
                })
                .property({
                    let property = PropertyHookConfig::builder()
                        .str_prefix(widestring::u16str!("💢").as_slice())
                        .system(
                            PropertySystemHookConfig::builder()
                                .size_no_alwayskb(true)
                                .size_max_bar(true)
                                .build(),
                        );
                    #[cfg(feature = "everything")]
                    let property = property.size_from_everything(true);
                    property.build()
                })
                .log(log_path)
                .build(),
        )
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
