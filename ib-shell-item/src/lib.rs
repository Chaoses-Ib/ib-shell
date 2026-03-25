/*!
A library for operating file system files / Windows Shell items.
*/
#![cfg_attr(feature = "hook", feature(sync_unsafe_cell))]
use windows::{
    Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx},
    core::HRESULT,
};

#[cfg(feature = "hook")]
pub mod hook;
pub mod id_list;
mod item;
pub mod path;
pub mod property;
mod string;

pub use item::*;

pub fn init() -> HRESULT {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
}
