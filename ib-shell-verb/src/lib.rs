/*!
A library for handling of custom Windows Shell verbs (actions like `open`) and injecting them.
*/
#![feature(assert_matches)]
#![feature(sync_unsafe_cell)]
use std::path::Path;

use anyhow::Context;

#[cfg(feature = "hook")]
pub mod hook;
pub mod workspace;

pub trait OpenVerb: Send + Sync {
    fn handle(&self, path: &Path) -> Option<anyhow::Result<()>>;
}

pub fn open_verbs(path: &Path, verbs: &[Box<dyn OpenVerb>]) -> Option<anyhow::Result<()>> {
    for verb in verbs {
        if let Some(result) = verb.handle(path) {
            return Some(result);
        }
    }
    None
}

pub fn open(path: &Path, verbs: &[Box<dyn OpenVerb>]) -> anyhow::Result<()> {
    if let Some(r) = open_verbs(path, verbs) {
        return r;
    }
    open::that_detached(path).context("open")
}
