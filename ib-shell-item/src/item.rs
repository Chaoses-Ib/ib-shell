use num_enum::TryFromPrimitive;
use widestring::U16CString;
use windows::{
    Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{
            Common::ITEMIDLIST, SHCreateItemFromIDList, SIGDN, SIGDN_DESKTOPABSOLUTEEDITING,
            SIGDN_DESKTOPABSOLUTEPARSING, SIGDN_FILESYSPATH, SIGDN_NORMALDISPLAY,
            SIGDN_PARENTRELATIVE, SIGDN_PARENTRELATIVEEDITING, SIGDN_PARENTRELATIVEFORADDRESSBAR,
            SIGDN_PARENTRELATIVEFORUI, SIGDN_PARENTRELATIVEPARSING, SIGDN_URL,
        },
    },
    core::Result,
};

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
    /// [SHCreateItemFromIDList function (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromidlist)
    #[doc(alias = "from_pidl")]
    fn from_id_list(id_list: *const ITEMIDLIST) -> Result<IShellItem>;

    /// [IShellItem::GetDisplayName (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitem-getdisplayname)
    fn get_display_name(&self, name: ShellItemDisplayName) -> Result<U16CString>;
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
}
