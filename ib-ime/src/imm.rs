use bitflags::bitflags;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::Input::{
        Ime::{
            ImmGetContext, ImmGetConversionStatus, ImmGetOpenStatus, ImmReleaseContext,
            ImmSetConversionStatus, ImmSetOpenStatus,
        },
        KeyboardAndMouse::GetFocus,
    },
};

/// `true` for on, `false` for off.
pub type ImeState = bool;

/// Get current IME state for a window.
///
/// Ref: https://github.com/dotnet/wpf/blob/aa997926b405d7ccea7a28f3b02ef8c1409ed4ca/src/Microsoft.DotNet.Wpf/src/PresentationCore/System/Windows/Input/InputMethod.cs#L538-L542
pub fn get_window_ime_state(hwnd: HWND) -> ImeState {
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_null() {
        return false;
    }

    let f_open = unsafe { ImmGetOpenStatus(himc) } != 0;
    unsafe { ImmReleaseContext(hwnd, himc) };

    f_open
}

/// Get current IME state for the window with focus.
pub fn get_ime_state() -> ImeState {
    let hwnd = unsafe { GetFocus() };
    get_window_ime_state(hwnd)
}

/// Set current IME state for a window.
///
/// Ref: https://github.com/dotnet/wpf/blob/aa997926b405d7ccea7a28f3b02ef8c1409ed4ca/src/Microsoft.DotNet.Wpf/src/PresentationCore/System/Windows/Input/InputMethod.cs#L571-L586
pub fn set_window_ime_state(hwnd: HWND, state: ImeState) {
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_null() {
        return;
    }

    let fopen = unsafe { ImmGetOpenStatus(himc) } != 0;

    // we don't have to call IMM unless the value is changed.
    if fopen != state {
        unsafe { ImmSetOpenStatus(himc, state as _) };
    }

    unsafe { ImmReleaseContext(hwnd, himc) };
}

/// Set current IME state for the window with focus.
pub fn set_ime_state(state: ImeState) {
    let hwnd = unsafe { GetFocus() };
    set_window_ime_state(hwnd, state);
}

/// IME conversion mode bitflags.
///
/// Ref: [IME Conversion Mode Values - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/intl/ime-conversion-mode-values)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImeConversionMode(u32);

bitflags! {
    impl ImeConversionMode: u32 {
        /// Alphanumeric input mode. This is the default, defined as 0x0000.
        const ALPHANUMERIC = 0x0000;
        /// Set to 1 if character code input mode; 0 if not.
        const CHARCODE = 0x0001;
        /// Set to 1 if EUDC conversion mode; 0 if not.
        const EUDC = 0x0002;
        /// Windows Me/98, Windows 2000, Windows XP: Set to 1 if fixed conversion mode; 0 if not.
        const FIXED = 0x0004;
        /// Set to 1 if full shape mode; 0 if half shape mode.
        const FULLSHAPE = 0x0008;
        /// Set to 1 if HANJA convert mode; 0 if not.
        const HANJACONVERT = 0x0010;
        /// Set to 1 if KATAKANA mode; 0 if HIRAGANA mode.
        const KATAKANA = 0x0020;
        /// Set to 1 if NATIVE mode; 0 if ALPHANUMERIC mode.
        const NATIVE = 0x0040;
        /// Set to 1 to prevent processing of conversions by IME; 0 if not.
        const NOCONVERSION = 0x0080;
        /// Set to 1 if ROMAN input mode; 0 if not.
        const ROMAN = 0x0100;
        /// Set to 1 if Soft Keyboard mode; 0 if not.
        const SOFTKBD = 0x0200;
        /// Set to 1 if SYMBOL conversion mode; 0 if not.
        const SYMBOL = 0x0400;
    }
}

/// Get current IME conversion mode for a window.
///
/// Ref: https://github.com/dotnet/wpf/blob/aa997926b405d7ccea7a28f3b02ef8c1409ed4ca/src/Microsoft.DotNet.Wpf/src/PresentationCore/System/Windows/Input/InputMethod.cs#L773-L775
pub fn get_window_ime_conversion_mode(hwnd: HWND) -> ImeConversionMode {
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_null() {
        return ImeConversionMode::empty();
    }

    let mut convmode = 0u32;
    let mut sentence = 0u32;
    unsafe { ImmGetConversionStatus(himc, &mut convmode, &mut sentence) };
    unsafe { ImmReleaseContext(hwnd, himc) };

    ImeConversionMode::from_bits_retain(convmode)
}

/// Get current IME conversion mode for the window with focus.
pub fn get_ime_conversion_mode() -> ImeConversionMode {
    let hwnd = unsafe { GetFocus() };
    get_window_ime_conversion_mode(hwnd)
}

/// Set current IME conversion mode for a window.
///
/// Ref: https://github.com/dotnet/wpf/blob/aa997926b405d7ccea7a28f3b02ef8c1409ed4ca/src/Microsoft.DotNet.Wpf/src/PresentationCore/System/Windows/Input/InputMethod.cs#L911-L977
pub fn set_window_ime_conversion_mode(hwnd: HWND, convmode: ImeConversionMode) {
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_null() {
        return;
    }

    unsafe { ImmSetConversionStatus(himc, convmode.bits(), 0) };
    unsafe { ImmReleaseContext(hwnd, himc) };
}

/// Set current IME conversion mode for the window with focus.
pub fn set_ime_conversion_mode(convmode: ImeConversionMode) {
    let hwnd = unsafe { GetFocus() };
    set_window_ime_conversion_mode(hwnd, convmode);
}
