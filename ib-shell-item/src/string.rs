use std::{mem, ptr};

use widestring::U16CStr;
use windows::{
    Win32::{
        System::Com::{CoTaskMemAlloc, CoTaskMemFree, CoTaskMemRealloc},
        UI::Shell::Common::{STRRET, STRRET_WSTR},
    },
    core::PWSTR,
};

#[allow(unused)]
pub fn strret_free(s: STRRET) {
    if s.uType == STRRET_WSTR.0 as u32 {
        unsafe { CoTaskMemFree(Some(s.Anonymous.pOleStr.0 as *mut _)) };
    }
}

/// Converts a `&str` to a `PWSTR` by allocating memory using `CoTaskMemAlloc`.
#[allow(unused)]
pub fn str_to_co_task(s: &str) -> PWSTR {
    let len = s.encode_utf16().count() + 1;

    let ptr = unsafe { CoTaskMemAlloc(len * mem::size_of::<u16>()) };
    if ptr.is_null() {
        return PWSTR(ptr::null_mut());
    }

    let ptr = ptr as *mut u16;
    unsafe {
        for (i, ch) in s.encode_utf16().enumerate() {
            ptr.add(i).write(ch);
        }
        ptr.add(len - 1).write(0);
    }

    PWSTR(ptr)
}

/// Prepends a prefix to a name slice and writes to ptr using CoTaskMemRealloc.
/// The name slice includes the original name with its null terminator.
pub(crate) fn prefix_u16cstr_ptr(name: &U16CStr, prefix: &[u16]) -> PWSTR {
    let ptr = unsafe {
        CoTaskMemRealloc(
            Some(name.as_ptr().cast()),
            (prefix.len() + name.len() + 1) * size_of::<u16>(),
        )
    } as *mut u16;

    if ptr.is_null() {
        return PWSTR(name.as_ptr() as _);
    }

    unsafe {
        ptr::copy_nonoverlapping(prefix.as_ptr(), ptr, prefix.len());
        ptr::copy_nonoverlapping(name.as_ptr(), ptr.add(prefix.len()), name.len() + 1);
    }

    PWSTR(ptr)
}
