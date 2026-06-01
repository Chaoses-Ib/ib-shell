# ib-shell
Some desktop environment libraries, mainly for Windows Shell (Windows' built-in desktop environment).

## [ib-shell-item](ib-shell-item/README.md)
[![crates.io](https://img.shields.io/crates/v/ib-shell-item.svg)](https://crates.io/crates/ib-shell-item)
[![Documentation](https://docs.rs/ib-shell-item/badge.svg)](https://docs.rs/ib-shell-item)
[![License](https://img.shields.io/crates/l/ib-shell-item.svg)](LICENSE.txt)

A library for operating file system files / Windows Shell items.

## [ib-shell-verb](ib-shell-verb/README.md)
[![crates.io](https://img.shields.io/crates/v/ib-shell-verb.svg)](https://crates.io/crates/ib-shell-verb)
[![Documentation](https://docs.rs/ib-shell-verb/badge.svg)](https://docs.rs/ib-shell-verb)
[![License](https://img.shields.io/crates/l/ib-shell-verb.svg)](LICENSE.txt)

A library for handling of custom Windows Shell verbs (actions like `open`) and injecting them.

### CLI
- `ib-open-workspace`:
  Given a file path, open its parent folder (or Git root) and show the file in VS Code.

  For example:
  ```sh
  ib-open-workspace --vscode README.md
  ```
  Works on Windows and Linux.

  If you don't want a binary, there is also a
  [cmd + VBScript version](ib-shell-verb/OpenFileInWorkspace/OpenFileInWorkspace.vbs).

## See also
### Integrations
- [IbEverythingExt: Everything 拼音搜索, ローマ字検索, wildcard, quick select, Shell extension](https://github.com/Chaoses-Ib/IbEverythingExt)

### Related projects
- [ib-ime: A library for controlling input controls' IME (Input Method Editor) behavior on Windows](https://github.com/Chaoses-Ib/ib-input#ib-ime)
- [SharpShell: Make it easy to create Windows Shell Extensions using the .NET Framework.](https://github.com/dwmkerr/sharpshell)
