use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
};

use anyhow::Context;
use bon::Builder;

use crate::OpenVerb;

#[derive(Builder)]
pub struct OpenFileInWorkspace {
    parent_as_workspace: bool,
    vscode: Option<OnceLock<PathBuf>>,
}

impl OpenFileInWorkspace {
    fn find_git_repo(p: &Path) -> Option<&Path> {
        for p in p.ancestors() {
            if p.join(".git").exists() {
                return Some(p);
            }
        }
        None
    }

    fn find_workspace<'p>(&self, p: &'p Path) -> Option<&'p Path> {
        if let Some(p) = Self::find_git_repo(p) {
            return Some(p);
        }
        if self.parent_as_workspace {
            return p.parent();
        }
        None
    }

    fn find_vscode() -> PathBuf {
        if let Ok(p) = which::which_global("code") {
            // e.g. C:\Users\Ib\AppData\Local\Programs\Microsoft VS Code\bin\code.cmd
            /*
            if p.extension().is_some_and(|ext| ext == "cmd") {
                let exe = p
                    .parent()
                    .and_then(Path::parent)
                    .map(|p| p.join("Code.exe"));
                if let Some(exe) = exe
                    && exe.exists()
                {
                    return exe;
                }
            }
            */
            // 0 syscall
            if p.file_name().is_some_and(|name| name == "code.cmd") {
                let exe = p
                    .parent()
                    .and_then(Path::parent)
                    .map(|p| p.join("Code.exe"));
                if let Some(exe) = exe {
                    return exe;
                }
            }
            return p;
        }
        "code".into()
    }
}

impl OpenVerb for OpenFileInWorkspace {
    fn handle(&self, path: &Path) -> Option<Result<(), anyhow::Error>> {
        let workspace = self.find_workspace(path)?;
        if let Some(ref vscode) = self.vscode {
            let vscode = vscode.get_or_init(Self::find_vscode);
            let r = Command::new(vscode)
                .arg("-n")
                .arg(workspace)
                .arg("-g")
                .arg(path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map(|_| ());
            return Some(r.context("vscode"));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_git_repo_none() {
        // Use CARGO_MANIFEST_PATH to get a known non-git directory
        let manifest = std::env::var("CARGO_MANIFEST_PATH").unwrap();
        let manifest_dir = Path::new(&manifest).parent().unwrap();

        // Find a directory that's definitely not a git repo
        // Start from the manifest directory and go up
        for ancestor in manifest_dir.ancestors() {
            let git_path = ancestor.join(".git");
            if git_path.exists() {
                // Found a git repo, look at its parent instead
                let result = OpenFileInWorkspace::find_git_repo(ancestor.parent().unwrap());
                assert!(result.is_none(), "Expected none for non-git directory");
                return;
            }
        }
        // If no git repo found at all, use the root itself
        let result = OpenFileInWorkspace::find_git_repo(manifest_dir);
        assert!(result.is_none(), "Expected none for non-git directory");
    }

    #[test]
    fn find_git_repo_some() {
        let manifest = std::env::var("CARGO_MANIFEST_PATH").unwrap();
        let manifest_dir = Path::new(&manifest).parent().unwrap();

        // Look for any git repository in ancestors
        for ancestor in manifest_dir.ancestors() {
            let git_path = ancestor.join(".git");
            if git_path.exists() {
                // Test from a file deep in the repo
                let deep_path = ancestor.join("src").join("workspace.rs");
                let result = OpenFileInWorkspace::find_git_repo(deep_path.parent().unwrap());
                assert_eq!(
                    result.map(|p| p.to_path_buf()),
                    Some(ancestor.to_path_buf())
                );
                return;
            }
        }
    }

    #[test]
    fn find_vscode() {
        let p = OpenFileInWorkspace::find_vscode();
        dbg!(p);
    }
}
