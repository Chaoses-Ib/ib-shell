use std::path::PathBuf;

use clap::Parser;

use ib_shell_verb::workspace::OpenFileInWorkspace;

#[derive(Parser)]
#[command(name = "ib-open-workspace")]
#[command(about = "Given a file path, open its parent folder (or Git root) and show the file in VS Code.", long_about = None)]
struct Args {
    /// Path to the file to open
    path: PathBuf,

    /// Open in VSCode
    #[arg(long)]
    vscode: bool,
}

fn main() {
    let args = Args::parse();

    let verb = OpenFileInWorkspace::builder()
        .parent_as_workspace(true)
        .maybe_vscode(args.vscode.then(Default::default))
        .build();

    let result = ib_shell_verb::open(&args.path.into(), &[Box::new(verb)]);
    if let Err(e) = result {
        eprintln!("{}", e);
    }
}
