/*!
A library for operating file system files / Windows Shell items.
*/
#![cfg_attr(feature = "hook", feature(sync_unsafe_cell))]

#[cfg(feature = "hook")]
pub mod hook;
pub mod id_list;
mod item;
pub mod path;
mod string;

pub use item::*;
