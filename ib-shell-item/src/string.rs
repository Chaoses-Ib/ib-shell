use std::ptr;

use widestring::U16CStr;
use windows::{Win32::System::Com::CoTaskMemRealloc, core::PWSTR};

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
