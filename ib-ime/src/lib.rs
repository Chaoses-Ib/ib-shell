/*!
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
[examples/winio.rs](../examples/winio.rs)

See also:
- [ib-matcher: A multilingual, flexible and fast string, glob and regex matcher. Support 拼音匹配 and ローマ字検索.](https://github.com/Chaoses-Ib/ib-matcher)
*/

pub mod hook;
pub mod imm;
