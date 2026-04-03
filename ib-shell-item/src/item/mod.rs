use std::cmp;

use num_enum::TryFromPrimitive;
use widestring::U16CString;
use windows::{
    Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{
            Common::ITEMIDLIST, SHCreateItemFromIDList, SHCreateItemFromParsingName, SIGDN,
            SIGDN_DESKTOPABSOLUTEEDITING, SIGDN_DESKTOPABSOLUTEPARSING, SIGDN_FILESYSPATH,
            SIGDN_NORMALDISPLAY, SIGDN_PARENTRELATIVE, SIGDN_PARENTRELATIVEEDITING,
            SIGDN_PARENTRELATIVEFORADDRESSBAR, SIGDN_PARENTRELATIVEFORUI,
            SIGDN_PARENTRELATIVEPARSING, SIGDN_URL,
        },
    },
    core::{PCWSTR, Result},
};

#[cfg(feature = "prop")]
pub mod item2;

pub use windows::Win32::UI::Shell::IShellItem;

/// Requests the form of an item's display name to retrieve through [`IShellItem::GetDisplayName`] and [`SHGetNameFromIDList`].
///
/// [SIGDN (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-sigdn)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(i32)]
pub enum ShellItemDisplayName {
    /// 0x00000000. Returns the display name relative to the parent folder.
    ///
    /// In UI this name is generally ideal for display to the user.
    NormalDisplay = SIGDN_NORMALDISPLAY.0,

    /// 0x80018001. Returns the parsing name relative to the parent folder.
    ///
    /// This name is not suitable for use in UI.
    ParentRelativeParsing = SIGDN_PARENTRELATIVEPARSING.0,

    /// 0x80028000. Returns the parsing name relative to the desktop.
    ///
    /// This name is not suitable for use in UI.
    DesktopAbsoluteParsing = SIGDN_DESKTOPABSOLUTEPARSING.0,

    /// 0x80031001. Returns the editing name relative to the parent folder.
    ///
    /// In UI this name is suitable for display to the user.
    ParentRelativeEditing = SIGDN_PARENTRELATIVEEDITING.0,

    /// 0x8004c000. Returns the editing name relative to the desktop.
    ///
    /// In UI this name is suitable for display to the user.
    DesktopAbsoluteEditing = SIGDN_DESKTOPABSOLUTEEDITING.0,

    /// 0x80058000. Returns the item's file system path, if it has one.
    ///
    /// Only items that report [`SFGAO_FILESYSTEM`] have a file system path. When an item does not have a file system path, a call to [`IShellItem::GetDisplayName`] on that item will fail.
    ///
    /// In UI this name is suitable for display to the user in some cases, but note that it might not be specified for all items.
    ///
    /// e.g. `C:\Users\Ib`
    FileSystemPath = SIGDN_FILESYSPATH.0,

    /// 0x80068000. Returns the item's URL, if it has one.
    ///
    /// Some items do not have a URL, and in those cases a call to [`IShellItem::GetDisplayName`] will fail.
    ///
    /// This name is suitable for display to the user in some cases, but note that it might not be specified for all items.
    Url = SIGDN_URL.0,

    /// 0x8007c001. Returns the path relative to the parent folder in a friendly format as displayed in an address bar.
    ///
    /// This name is suitable for display to the user.
    ParentRelativeForAddressBar = SIGDN_PARENTRELATIVEFORADDRESSBAR.0,

    /// 0x80080001. Returns the path relative to the parent folder.
    ParentRelative = SIGDN_PARENTRELATIVE.0,

    /// 0x80094001. Introduced in Windows 8. Returns the path relative to the parent folder for UI purposes.
    ParentRelativeForUI = SIGDN_PARENTRELATIVEFORUI.0,
}

impl ShellItemDisplayName {
    /// Returns `true` if this display name is meant for parsing.
    pub fn is_for_parse(&self) -> bool {
        use ShellItemDisplayName::*;
        matches!(
            self,
            ParentRelativeParsing | DesktopAbsoluteParsing | FileSystemPath | Url | ParentRelative
        )
    }

    /// Returns `true` if this display name is meant for displaying in UI.
    pub fn is_for_display(&self) -> bool {
        use ShellItemDisplayName::*;
        matches!(
            self,
            NormalDisplay | ParentRelativeForAddressBar | ParentRelativeForUI
        )
    }

    /// Returns `true` if this display name is meant for editing in UI.
    pub fn is_for_edit(&self) -> bool {
        use ShellItemDisplayName::*;
        matches!(self, ParentRelativeEditing | DesktopAbsoluteEditing)
    }
}

/// [IShellItem (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ishellitem)
pub trait ShellItem {
    /// [SHCreateItemFromParsingName function (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromparsingname)
    ///
    /// Although not documented, this requires `CoInitialize()`.
    fn from_path_w(path: PCWSTR) -> Result<IShellItem> {
        unsafe { SHCreateItemFromParsingName::<_, _, IShellItem>(path, None) }
    }

    /// [SHCreateItemFromIDList function (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromidlist)
    #[doc(alias = "from_pidl")]
    fn from_id_list(id_list: *const ITEMIDLIST) -> Result<IShellItem>;

    /// [IShellItem::GetDisplayName (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitem-getdisplayname)
    fn get_display_name(&self, name: ShellItemDisplayName) -> Result<U16CString>;

    /// - `flags`: [`SICHINTF_*` (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_sichintf)
    ///
    ///   Different from [`ShellFolder::compare_ids()`], specifying the column doesn't work.
    ///   (Cleared by `& 0xf0000000`)
    ///
    /// Internally, this usually calls [`ShellFolder::compare_ids()`].
    ///
    /// [IShellItem::Compare (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitem-compare)
    ///
    /// ## Returns
    /// Although the doc just says
    /// "If the two items are the same this parameter equals zero; if they are different the parameter is nonzero."
    /// The sign actually defines the order like [`ShellFolder::compare_ids()`] does.
    ///
    /// [`ShellFolder::compare_ids()`]: crate::folder::ShellFolder::compare_ids
    fn compare(&self, psi: &IShellItem, flags: u32) -> Result<cmp::Ordering>;
}

impl ShellItem for IShellItem {
    /// Ref: https://github.com/Hau-Hau/restart-explorer/blob/230ed6dd78ac656a86e07310c3afc62f03057a36/src/infrastructure/windows_os/persist_id_list.rs
    fn from_id_list(id_list: *const ITEMIDLIST) -> Result<IShellItem> {
        unsafe { SHCreateItemFromIDList::<IShellItem>(id_list) }
    }

    /// Ref: https://github.com/Hau-Hau/restart-explorer/blob/230ed6dd78ac656a86e07310c3afc62f03057a36/src/infrastructure/windows_os/shell_item.rs
    fn get_display_name(&self, name: ShellItemDisplayName) -> Result<U16CString> {
        let name = unsafe { self.GetDisplayName(SIGDN(name as i32)) }?;
        let name_u16 = unsafe { U16CString::from_ptr_str(name.0) };
        unsafe { CoTaskMemFree(Some(name.0 as _)) };
        Ok(name_u16)
    }

    fn compare(&self, psi: &IShellItem, hint: u32) -> Result<cmp::Ordering> {
        let order = unsafe { self.Compare(psi, hint)? };
        Ok(order.cmp(&0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::w;

    use crate::{init, prop::column::FSColumn};

    #[test]
    fn compare() {
        _ = init();
        let windows = IShellItem::from_path_w(w!(r"C:\Windows")).unwrap();
        let users = IShellItem::from_path_w(w!(r"C:\Users")).unwrap();

        let result = windows.compare(&users, 0).unwrap();
        assert_eq!(result, cmp::Ordering::Greater);
        assert_eq!(users.compare(&windows, 0).unwrap(), cmp::Ordering::Less);
    }

    #[test]
    fn compare_size() {
        _ = init();
        let explorer = IShellItem::from_path_w(w!(r"C:\Windows\explorer.exe")).unwrap();
        let notepad = IShellItem::from_path_w(w!(r"C:\Windows\notepad.exe")).unwrap();

        let result = explorer.compare(&notepad, 0).unwrap();
        assert_eq!(result, cmp::Ordering::Less);

        // Different from [`ShellFolder::compare_ids()`], specifying the column doesn't work.
        let result = explorer.compare(&notepad, FSColumn::Size as u32).unwrap();
        assert_eq!(result, cmp::Ordering::Less);
    }

    #[test]
    fn compare_same() {
        _ = init();
        let windows = IShellItem::from_path_w(w!(r"C:\Windows")).unwrap();
        let windows2 = IShellItem::from_path_w(w!(r"C:\Windows")).unwrap();

        let result = windows.compare(&windows2, 0).unwrap();
        assert_eq!(result, cmp::Ordering::Equal);
    }
}
