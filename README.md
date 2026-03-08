# ib-shell
Some desktop environment libraries, mainly for Windows Shell (Windows' built-in desktop environment).

## [ib-ime](ib-ime/README.md)
[![crates.io](https://img.shields.io/crates/v/ib-ime.svg)](https://crates.io/crates/ib-ime)
[![Documentation](https://docs.rs/ib-ime/badge.svg)](https://docs.rs/ib-ime)
[![License](https://img.shields.io/crates/l/ib-ime.svg)](LICENSE.txt)

A library for controlling input controls' IME (Input Method Editor) behavior on Windows.

Usage:
```rust
// cargo add ib-ime
// Manually set:
ib_ime::imm::set_ime_state(false);
ib_ime::imm::set_ime_conversion_mode(ib_ime::imm::ImeConversionMode::ALPHANUMERIC);

// Automatically turn off IME by default for a window (or an editor control):
ib_ime::hook::ImeHookConfig::default_off().hook_window(edit_hwnd);
```

[Winio](https://github.com/compio-rs/winio) integration example:
[examples/winio.rs](ib-ime/examples/winio.rs)

See also:
- [ib-matcher: A multilingual, flexible and fast string, glob and regex matcher. Support 拼音匹配 and ローマ字検索.](https://github.com/Chaoses-Ib/ib-matcher)

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
- [SharpShell: Make it easy to create Windows Shell Extensions using the .NET Framework.](https://github.com/dwmkerr/sharpshell)
