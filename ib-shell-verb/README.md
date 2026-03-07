# ib-shell-verb
[![crates.io](https://img.shields.io/crates/v/ib-shell-verb.svg)](https://crates.io/crates/ib-shell-verb)
[![Documentation](https://docs.rs/ib-shell-verb/badge.svg)](https://docs.rs/ib-shell-verb)
[![License](https://img.shields.io/crates/l/ib-shell-verb.svg)](../LICENSE.txt)

A library for handling of custom Windows Shell verbs (actions like `open`) and injecting them.

## CLI
- `ib-open-workspace`:
  Given a file path, open its parent folder (or Git root) and show the file in VS Code.

  For example:
  ```sh
  ib-open-workspace --vscode README.md
  ```
  Works on Windows and Linux.

  If you don't want a binary, there is also a
  [cmd + VBScript version](OpenFileInWorkspace/OpenFileInWorkspace.vbs).
