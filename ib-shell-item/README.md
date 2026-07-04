# ib-shell-item
[![crates.io](https://img.shields.io/crates/v/ib-shell-item.svg)](https://crates.io/crates/ib-shell-item)
[![Documentation](https://docs.rs/ib-shell-item/badge.svg)](https://docs.rs/ib-shell-item)
[![License](https://img.shields.io/crates/l/ib-shell-item.svg)](../LICENSE.txt)

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

Hooking preview:  
![](../docs/images/item-hook.png)
