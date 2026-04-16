/*!
A library for handling of custom Windows Shell verbs (actions like `open`) and injecting them.

## Features
*/
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "doc", doc = document_features::document_features!())]
// #![cfg_attr(test, feature(assert_matches))]
#![cfg_attr(feature = "hook", feature(sync_unsafe_cell))]
use std::path::Path;

use anyhow::{Context, bail};
use ib_shell_item::path::ShellPath;

#[cfg(feature = "hook")]
pub mod hook;
pub mod workspace;

pub trait OpenVerb: Send + Sync {
    fn handle(&self, path: &Path) -> Option<anyhow::Result<()>>;

    fn handle_shell(&self, path: &ShellPath) -> Option<anyhow::Result<()>> {
        self.handle(&path.to_file_path().ok()?)
    }
}

pub fn open_verbs(path: &ShellPath, verbs: &[Box<dyn OpenVerb>]) -> Option<anyhow::Result<()>> {
    for verb in verbs {
        if let Some(result) = verb.handle_shell(path) {
            return Some(result);
        }
    }
    None
}

pub fn open(path: &ShellPath, verbs: &[Box<dyn OpenVerb>]) -> anyhow::Result<()> {
    if let Some(r) = open_verbs(path, verbs) {
        return r;
    }
    if let Ok(path) = path.to_file_path() {
        open::that_detached(path).context("open")
    } else {
        bail!("TODO")
    }
}
