use std::ptr;

use widestring::U16CStr;
use windows::{
    Win32::{
        Foundation::PROPERTYKEY,
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, StructuredStorage::PROPVARIANT},
        UI::Shell::PropertiesSystem::{PROPDESC_FORMAT_FLAGS, PSGetPropertySystem},
    },
    core::{Interface, PWSTR, Result},
};

pub use windows::Win32::UI::Shell::PropertiesSystem::IPropertySystem;

/// [IPropertySystem (propsys.h)](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nn-propsys-ipropertysystem)
///
/// Exposes methods that get property descriptions, register and unregister property schemas, enumerate property descriptions, and format property values in a type-strict way.
pub trait PropertySystem {
    /// Call `CoInitialize()` before [`PropertySystem::new()`].
    fn new_init() -> Result<IPropertySystem>;

    /// [PSGetPropertySystem function (propsys.h)](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-psgetpropertysystem)
    fn new() -> Result<IPropertySystem>;

    /// [IPropertySystem::FormatForDisplay (propsys.h)](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-ipropertysystem-formatfordisplay)
    ///
    /// Formats a property value for display into a user-provided buffer.
    ///
    /// ## Arguments
    /// - `key`: The property key to format
    /// - `propvar`: The property value to format
    /// - `pdff`: Format flags
    /// - `buffer`: Mutable buffer to receive the formatted string (UTF-16)
    fn format_for_display<'b>(
        &self,
        key: &PROPERTYKEY,
        propvar: &PROPVARIANT,
        pdff: PROPDESC_FORMAT_FLAGS,
        buffer: &'b mut [u16],
    ) -> Result<&'b mut U16CStr>;

    /// [IPropertySystem::FormatForDisplayAlloc (propsys.h)](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-ipropertysystem-formatfordisplayalloc)
    ///
    /// ## Arguments
    /// - `key`: The property key to format
    ///
    ///   Can be controlled by [`displayInfo`](https://learn.microsoft.com/en-us/windows/win32/properties/propdesc-schema-displayinfo).
    /// - `propvar`: The property value to format
    /// - `pdff`: Format flags
    ///
    /// ## Returns
    /// The calling application must use [`CoTaskMemFree`] to release the returned string when it is no longer needed.
    ///
    /// ## Implementation
    /// ```
    /// CFormatForDisplay::FormatForDisplay() {
    ///   CFormatForDisplay::_FormatSize() {
    ///     PSStrFormatKBSizeW()
    ///   }
    /// }
    /// ```
    fn format_for_display_alloc(
        &self,
        key: &PROPERTYKEY,
        propvar: &PROPVARIANT,
        pdff: PROPDESC_FORMAT_FLAGS,
    ) -> Result<PWSTR>;
}

impl PropertySystem for IPropertySystem {
    fn new_init() -> Result<IPropertySystem> {
        _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self::new()
    }

    fn new() -> Result<IPropertySystem> {
        let mut pv = ptr::null_mut();
        unsafe { PSGetPropertySystem(&IPropertySystem::IID, &mut pv)? };
        Ok(unsafe { IPropertySystem::from_raw(pv) })
    }

    fn format_for_display<'b>(
        &self,
        key: &PROPERTYKEY,
        propvar: &PROPVARIANT,
        pdff: PROPDESC_FORMAT_FLAGS,
        buffer: &'b mut [u16],
    ) -> Result<&'b mut U16CStr> {
        unsafe { self.FormatForDisplay(key, propvar, pdff, buffer) }?;
        Ok(U16CStr::from_slice_truncate_mut(buffer).unwrap())
    }

    fn format_for_display_alloc(
        &self,
        key: &PROPERTYKEY,
        propvar: &PROPVARIANT,
        pdff: PROPDESC_FORMAT_FLAGS,
    ) -> Result<PWSTR> {
        unsafe { self.FormatForDisplayAlloc(key, propvar, pdff) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use windows::Win32::Storage::EnhancedStorage::PKEY_Size;

    #[test]
    fn format_for_display() {
        let prop_system = IPropertySystem::new_init().expect("Failed to create IPropertySystem");
        let key = PKEY_Size;
        let propvar = PROPVARIANT::from(1024_i64);
        let pdff = PROPDESC_FORMAT_FLAGS::default();

        // Create a buffer to receive the formatted string
        let mut buffer: [u16; 256] = [0; 256];

        let display_value = prop_system
            .format_for_display(&key, &propvar, pdff, &mut buffer)
            .expect("Failed to format for display");
        assert_eq!(display_value, widestring::u16cstr!("1.00 KB"));
    }

    #[test]
    fn format_for_display_alloc_with_pkey_size() {
        let prop_system = IPropertySystem::new_init().expect("Failed to create IPropertySystem");

        // Use PKEY_Size as the key
        let key = PKEY_Size;

        // Create a PROPVARIANT from a number (e.g., 1024 for 1KB)
        let propvar = PROPVARIANT::from(1024_i64);

        // Format with default flags
        let pdff = PROPDESC_FORMAT_FLAGS::default();

        let result = prop_system
            .format_for_display_alloc(&key, &propvar, pdff)
            .expect("Failed to format for display");

        // Convert PWSTR to Rust string for verification
        let display_value: String =
            unsafe { result.to_string() }.expect("Failed to convert PWSTR to String");

        // Assert with expected format (e.g., "1 KB", "1,024 bytes", etc.)
        assert!(!display_value.is_empty());
        // The exact format depends on system locale and format flags
        // Common formats: "1 KB", "1,024 bytes", "1,024 B", etc.
        // Log the actual format for reference
        eprintln!("Formatted size value: {display_value}");

        // For 1024 bytes, the expected format is "1.00 KB" (with default flags)
        assert_eq!(display_value, "1.00 KB");
    }

    #[test]
    fn format_for_display_alloc_with_various_sizes() {
        let prop_system = IPropertySystem::new_init().expect("Failed to create IPropertySystem");

        let key = PKEY_Size;

        let test_cases = [
            (1024_i64, "1.00 KB"),
            (1024 * 1024, "1.00 MB"),
            (1024 * 1024 * 1024, "1.00 GB"),
        ];

        for (size, expected) in test_cases {
            let propvar = PROPVARIANT::from(size);
            let result = prop_system
                .format_for_display_alloc(&key, &propvar, PROPDESC_FORMAT_FLAGS::default())
                .expect("Failed to format for display");

            let display_value: String =
                unsafe { result.to_string() }.expect("Failed to convert PWSTR");
            assert_eq!(
                display_value, expected,
                "Size {size} should format to '{expected}'"
            );
        }
    }
}
