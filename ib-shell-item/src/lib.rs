/*!
A library for operating file system files / Windows Shell items.

## Features
- Shell item ID list (path): `AbsoluteIDList`, `RelativeIDList`, `ChildID` and their `Ref` variants; `IPersistIDList`.
- Shell folder: `IShellFolder`.
- Shell item: `IShellItem`, `IShellItem2`.
- Property System: `IPropertySystem`, `IPropertyStore`, `FSColumn`.
- Hooking
  - Hook item display name.
  - Display folder size with bar graph from Everything.
- COM string utilities.

## Crate features
*/
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "doc", doc = document_features::document_features!())]
#![cfg_attr(feature = "hook", feature(sync_unsafe_cell))]
use windows::{
    Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx},
    core::HRESULT,
};

pub mod folder;
#[cfg(feature = "hook")]
pub mod hook;
pub mod id_list;
mod item;
pub mod path;
pub mod prop;
pub mod string;

pub use item::*;

pub fn init() -> HRESULT {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
}
