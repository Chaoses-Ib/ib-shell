/*!
## `CompareIDs()`
[IShellFolder::CompareIDs (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-compareids)

In Explorer of Windows 11 24H2, `CompareIDs()` is called through:
```cpp
windows.storage.dll!DVCompareColumns+0x1d7
windows.storage.dll!_DSA_MergeSort2+0x15e
windows.storage.dll!_DSA_MergeSort2+0x54
windows.storage.dll!_DSA_MergeSort2+0x54
windows.storage.dll!_DSA_MergeSort2+0x66
windows.storage.dll!_DSA_MergeSort+0x69
windows.storage.dll!DSA_Sort+0x4f
windows.storage.dll!CSortTask::InternalResumeRT+0xc0
windows.storage.dll!CRunnableTask::Run+0xb6
windows.storage.dll!CShellTaskThread::ThreadProc+0x2ca
windows.storage.dll!CShellTaskThread::s_ThreadProc+0x15e
SHCore.dll!ExecuteWorkItemThreadProc+0x15
```

Different PIDLs of the same file system path may be passed in rare cases.

If `CompareIDs()` returns error, Explorer will fall back to compare `ItemNameDisplay`
instead when comparing other properties, including folder size.

Inconsistent ordering may cause buggy results, like the column order is sometimes correct but sometimes wrong.

## Implementations
### `CFSFolder`
Interfaces (on Windows 11 24H2):
- `000214e6_0000_0000_c000_000000000046` `ShellFolder`
- `b3a4b685_b685_4805_99d9_5dead2873236` `ParentAndItem`
- `93f2f68c_1d1b_11d3_a30e_00c04f79abd1` `ShellFolder2`
- `cef04fdf_fe72_11d2_87a5_00c04f6837cf` `PersistFolder3`
- `0000010c_0000_0000_c000_000000000046` `Persist`
- `000214ea_0000_0000_c000_000000000046` `PersistFolder`
- `1ac3d9f0_175c_11d1_95be_00609797ea4f` `PersistFolder2`
- `000214e5_0000_0000_c000_000000000046` `ShellIcon`
- `add8ba80_002b_11d0_8f0f_00c04fd7d062` `DelegateFolder`
- `321a6a6a_d61f_4bf3_97ae_14be2986bb36` `ObjectWithBackReferences`
- `7d688a70_c613_11d0_999b_00c04fd655e1` `ShellIconOverlay`
- `37d84f60_42cb_11ce_8135_00aa004bb851` `PersistPropertyBag`
- `0000000b_0000_0000_c000_000000000046` `Storage`
- `1df0d7f1_b267_4d28_8b10_12e23202a5c4` `ItemNameLimits`
- `3409e930_5a39_11d1_83fa_00a0c90dc849` `ContextMenuCB`
- `b722bccb_4e68_101b_a2bc_00aa00404770` `OleCommandTarget`
- `a6087428_3be3_4d73_b308_7c04a540bf1a` `ObjectProvider`
- `fc4801a3_2ba9_11cf_a229_00aa003d7352` `ObjectWithSite`
- `000214fe_0000_0000_c000_000000000046` `RemoteComputer`
- `e35b4b2e_00da_4bc1_9f13_38bc11f5d417` `ThumbnailHandlerFactory`
- `e07010ec_bc17_44c0_97b0_46c7c95b9edc` `ExplorerPaneVisibility`
- `e9701183_e6b3_4ff2_8568_813615fec7be` `NameSpaceTreeControlFolderCapabilities`
- `c938b119_d3ad_4d02_b5ee_164c2ec8160e`
- `fdbee76e_f12b_408e_93ab_9be8521000d9`
- `2536f9ac_2876_408a_9adf_1fe1c14c0e7f`
- `089f3011_bb5c_4f9c_9b8f_9a67ed446e91`
- `08727c66_4a04_456d_8c9a_cc1f65490753`
- `76347b91_9846_4ce7_9a57_69b910d16123`
- `0681c275_472b_4097_97b3_f19e4875fdc9`
- `124bae2c_cb94_42cd_b5b8_4358789684ef`
- `ff314a1e_06fa_4f3a_84be_7aa1c6be2470`
- `47d9e2b2_cbb3_4fe3_a925_f49978685982`
- `053b4a86_0dc9_40a3_b7ed_bc6a2e951f48`
- `3f943012_447b_4109_8b74_720106853c96`
- `c51e78b5_566b_4cb0_b6ed_784e18797e23`
- `dc0ac42a_141e_4876_9c43_824829440de0`
- `be9da82b_cc54_4b19_8c22_ad7762ff29eb`
- `013c437f_d523_41fa_8beb_f5100e1ca41c`
- `127f6acb_7e78_4368_83a4_ed1de72baca6`
- `d960050c_f4e1_4294_ac4b_598913605923`
*/
use std::{cmp, mem};

use bon::Builder;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM},
        UI::Shell::{
            Common::{ITEMIDLIST, STRRET},
            IShellFolder, SHCIDS_ALLFIELDS, SHCIDS_CANONICALONLY, SHGDN_FORPARSING, SHGDNF,
            SHGetDesktopFolder, StrRetToBSTR,
        },
    },
    core::{BSTR, PCWSTR, Result, w},
};

use crate::{
    id_list::{ChildIDRef, RelativeIDList},
    prop::attribute::ItemAttributes,
};

mod compare;

/// [IShellFolder::CompareIDs (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-compareids#parameters)
#[derive(Debug, Clone, Copy, Default, Builder)]
pub struct CompareIDs {
    /// - [`prop::column::FSColumn`](crate::prop::column::FSColumn)
    #[builder(default, into)]
    pub column: u16,
    #[builder(default)]
    pub flags: u16,
}

impl CompareIDs {
    /// Version 5.0.
    /// Compare all the information contained in the [`ITEMIDLIST`] structure, not just the display names.
    /// This flag is valid only for folder objects that support the [`IShellFolder2`] interface.
    /// For instance, if the two items are files, the folder should compare their names, sizes, file times, attributes, and any other information in the structures.
    /// If this flag is set, `column` must be zero.
    pub const ALL_FIELDS: CompareIDs = CompareIDs {
        column: 0,
        flags: (SHCIDS_ALLFIELDS >> 16) as u16,
    };

    /// Version 5.0.
    /// When comparing by name, compare the system names but not the display names.
    /// When this flag is passed, the two items are compared by whatever criteria the Shell folder determines are most efficient, as long as it implements a consistent sort function.
    /// This flag is useful when comparing for equality or when the results of the sort are not displayed to the user.
    /// This flag cannot be combined with other flags.
    pub const CANONICAL_ONLY: CompareIDs = CompareIDs {
        column: 0,
        flags: (SHCIDS_CANONICALONLY >> 16) as u16,
    };
}

impl Into<LPARAM> for CompareIDs {
    fn into(self) -> LPARAM {
        LPARAM((self.column as u32 | (self.flags as u32) << 16) as isize)
    }
}

/// [IShellFolder (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ishellfolder)
pub trait ShellFolder {
    /// Retrieves the [`IShellFolder`] interface for the desktop folder, which is the root of the Shell's namespace.
    ///
    /// [SHGetDesktopFolder (shlobj_core.h)](hhttps://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetdesktopfolder)
    fn from_desktop() -> Result<IShellFolder> {
        unsafe { SHGetDesktopFolder() }
    }

    /// Creates a child folder that represents the folder containing the given item.
    ///
    /// [IShellFolder::BindToObject (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-bindtoobject)
    fn from_id_list(pidl: &RelativeIDList) -> Result<IShellFolder> {
        let desktop = Self::from_desktop()?;
        unsafe { desktop.BindToObject(pidl.0, None) }
    }

    /// Creates a shell folder from a display name path.
    ///
    /// This combines [`ShellFolder::parse_display_name()`] and [`ShellFolder::from_id_list`] to parse a path and return its folder.
    ///
    /// Ref:
    /// - [IShellFolder from Path String](https://forums.codeguru.com/showthread.php?105564-IShellFolder-from-Path-String)
    /// - [php - How can I convert an absolute system path to an IShellFolder? - Stack Overflow](https://stackoverflow.com/questions/22548071/how-can-i-convert-an-absolute-system-path-to-an-ishellfolder)
    fn from_path_w(hwnd: HWND, path: PCWSTR) -> Result<IShellFolder> {
        let desktop = Self::from_desktop()?;
        let pidl = desktop.parse_display_name_to_id_list(hwnd, path)?;
        unsafe { desktop.BindToObject(pidl.0, None) }
    }

    /// Returns an arbitrary object of [`CFSFolder`](super::folder#cfsfolder).
    fn from_fs_any(hwnd: HWND) -> Result<IShellFolder> {
        Self::from_path_w(hwnd, w!(r"C:\Windows"))
    }

    /// Translates the display name of a file object or a folder into an item identifier list.
    ///
    /// - Doesn't handle relative path or parent folder indicators ("." or "..").
    /// - Case-insensitive.
    ///
    /// [IShellFolder::ParseDisplayName (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-parsedisplayname)
    ///
    /// ## Returns
    /// When it is no longer needed, it is the responsibility of the caller to free this resource by calling [`CoTaskMemFree`].
    fn parse_display_name(
        &self,
        hwnd: HWND,
        display_name: PCWSTR,
    ) -> Result<(usize, RelativeIDList)>;

    fn parse_display_name_to_id_list(
        &self,
        hwnd: HWND,
        display_name: PCWSTR,
    ) -> Result<RelativeIDList> {
        self.parse_display_name(hwnd, display_name).map(|r| r.1)
    }

    /// [IShellFolder::CompareIDs (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-compareids)
    ///
    /// See [`CompareIDs()`](super::folder#compareids) for details.
    fn compare_ids(
        &self,
        param: CompareIDs,
        pidl1: &RelativeIDList,
        pidl2: &RelativeIDList,
    ) -> Result<cmp::Ordering>;

    /// [IShellFolder::GetAttributesOf (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-getattributesof)
    ///
    /// ## Returns
    /// Requested attributes in `mask` that are common to all of the specified items.
    fn get_attributes_of(
        &self,
        children: &[ChildIDRef],
        mask: ItemAttributes,
    ) -> Result<ItemAttributes>;

    /// Tests if the child is a Shell folder (not necessarily a file system directory).
    ///
    /// This can also be implemented via [`ShellFolder::get_display_name_of()`].
    /// But it's probably slower as attributes are already stored in the ID.
    fn is_child_folder(&self, child: ChildIDRef) -> bool {
        self.get_attributes_of(&[child], ItemAttributes::Folder)
            .is_ok_and(|attrs| attrs.contains(ItemAttributes::Folder))
    }

    /// Tests if the child is a file system directory.
    ///
    /// This can also be implemented via [`ShellFolder::get_path_of()`].
    /// But it's probably slower as attributes are already stored in the ID.
    fn is_child_fs_folder(&self, child: ChildIDRef) -> bool {
        const MASK: ItemAttributes = ItemAttributes::Folder.union(ItemAttributes::FileSystem);
        self.get_attributes_of(&[child], MASK)
            .is_ok_and(|attrs| attrs.contains(MASK))
    }

    /// Tests if the children are Shell folders (not necessarily file system directories).
    ///
    /// This can also be implemented via [`ShellFolder::get_display_name_of()`].
    /// But it's probably slower as attributes are already stored in the ID.
    #[deprecated = "This is usually not supported, including CFSFolder from Windows XP to 11"]
    fn are_children_folders(&self, children: &[ChildIDRef]) -> bool {
        self.get_attributes_of(children, ItemAttributes::Folder)
            .is_ok_and(|attrs| attrs.contains(ItemAttributes::Folder))
    }

    /// Retrieves the display name for a specified item in the namespace.
    ///
    /// [IShellFolder::GetDisplayNameOf (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-getdisplaynameof)
    ///
    /// ## Parameters
    /// - `pidl`: Pointer to an item identifier list that identifies the child item.
    /// - `uflags`: Flags that specify the type of display name to return.
    ///
    ///   See [_SHGDNF (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_shgdnf)
    ///
    /// ## Returns
    /// The display name of the item specified by `pidl`, in the format specified by `uflags`.
    ///
    /// Because [`STRRET`] is hard to work with, this method converts it to [`BSTR`]
    /// before returning. However, this introduces a mem alloc. If you want to avoid it
    /// you can directly call [`IShellFolder::GetDisplayNameOf()`].
    fn get_display_name_of(&self, pidl: ChildIDRef, uflags: SHGDNF) -> Result<BSTR>;

    /// Get the display name for parsing relative to the desktop.
    ///
    /// i.e. `get_display_name_of(SHGDN_FORPARSING)`
    ///
    /// ## Returns
    /// e.g. `C:\Windows`
    fn get_path_of(&self, pidl: ChildIDRef) -> Result<BSTR> {
        self.get_display_name_of(pidl, SHGDN_FORPARSING)
    }
}

impl ShellFolder for IShellFolder {
    fn parse_display_name(
        &self,
        hwnd: HWND,
        display_name: PCWSTR,
    ) -> Result<(usize, RelativeIDList)> {
        let mut ch_eaten: u32 = 0;
        let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
        unsafe {
            self.ParseDisplayName(
                hwnd,
                None,
                display_name,
                Some(&mut ch_eaten),
                &mut pidl,
                std::ptr::null_mut(),
            )
        }?;
        Ok((ch_eaten as usize, RelativeIDList(pidl)))
    }

    fn compare_ids(
        &self,
        param: CompareIDs,
        pidl1: &RelativeIDList,
        pidl2: &RelativeIDList,
    ) -> Result<cmp::Ordering> {
        let hres = unsafe { self.CompareIDs(param.into(), pidl1.0, pidl2.0) };
        hres.ok()?;
        let code = (hres.0 & 0xFFFF) as i16;
        // dbg!(code, code.cmp(&0));
        Ok(code.cmp(&0))
    }

    fn get_attributes_of(
        &self,
        children: &[ChildIDRef],
        mask: ItemAttributes,
    ) -> Result<ItemAttributes> {
        let children: &[*const ITEMIDLIST] = unsafe { mem::transmute(children) };
        let mut mask = mask.bits();
        unsafe { self.GetAttributesOf(children, &mut mask) }?;
        Ok(ItemAttributes::from_bits_retain(mask))
    }

    fn get_display_name_of(&self, pidl: ChildIDRef, uflags: SHGDNF) -> Result<BSTR> {
        let mut name = STRRET::default();
        unsafe { self.GetDisplayNameOf(pidl.0, uflags, &mut name) }?;
        let mut str = BSTR::new();
        (unsafe { StrRetToBSTR(&mut name, Some(pidl.0), &mut str) })?;
        Ok(str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::w;

    use crate::prop::column::FSColumn;

    #[test]
    fn from_desktop() {
        let _desktop = IShellFolder::from_desktop().unwrap();
    }

    #[test]
    fn from_path() {
        let _folder = IShellFolder::from_path_w(HWND::default(), w!(r"C:\Windows")).unwrap();
    }

    #[test]
    fn parse_display_name() {
        let desktop = IShellFolder::from_desktop().unwrap();
        let display_name = w!(r"C:\Windows");
        let result = desktop.parse_display_name(HWND::default(), display_name);
        dbg!(&result);
        let (_ch_eaten, pidl) = result.unwrap();
        // Broken?
        // assert!(ch_eaten > 0);
        assert!(!pidl.0.is_null());
    }

    #[test]
    fn compare_ids() {
        let c = IShellFolder::from_path_w(HWND::default(), w!(r"C:\")).unwrap();

        let windows_pidl = c
            .parse_display_name_to_id_list(HWND::default(), w!(r"Windows"))
            .unwrap();
        let users_pidl = c
            .parse_display_name_to_id_list(HWND::default(), w!(r"Users"))
            .unwrap();
        let (windows_pidl, users_pidl) = (&windows_pidl, &users_pidl);
        dbg!(windows_pidl, users_pidl);

        let result = c
            .compare_ids(Default::default(), windows_pidl, users_pidl)
            .unwrap();
        assert_eq!(result, cmp::Ordering::Greater);

        let result = c
            .compare_ids(CompareIDs::CANONICAL_ONLY, windows_pidl, users_pidl)
            .unwrap();
        assert_eq!(result, cmp::Ordering::Greater);

        let result = c
            .compare_ids(
                CompareIDs::builder().column(FSColumn::Size).build(),
                windows_pidl,
                users_pidl,
            )
            .unwrap();
        assert_eq!(result, cmp::Ordering::Equal);
    }

    #[test]
    fn compare_ids_size() {
        let windows = IShellFolder::from_path_w(HWND::default(), w!(r"C:\Windows")).unwrap();

        let explorer_pidl = windows
            .parse_display_name_to_id_list(HWND::default(), w!(r"explorer.exe"))
            .unwrap();
        let notepad_pidl = windows
            .parse_display_name_to_id_list(HWND::default(), w!(r"notepad.exe"))
            .unwrap();
        let (explorer_pidl, notepad_pidl) = (&explorer_pidl, &notepad_pidl);

        let result = windows
            .compare_ids(
                CompareIDs::builder().column(FSColumn::Size).build(),
                explorer_pidl,
                notepad_pidl,
            )
            .unwrap();
        assert_eq!(result, cmp::Ordering::Greater);
        let result = windows
            .compare_ids(
                CompareIDs::builder().column(FSColumn::Size).build(),
                notepad_pidl,
                explorer_pidl,
            )
            .unwrap();
        assert_eq!(result, cmp::Ordering::Less);
    }

    #[test]
    fn compare_ids_nest() {
        let desktop = IShellFolder::from_desktop().unwrap();

        let windows_pidl = desktop
            .parse_display_name_to_id_list(HWND::default(), w!(r"C:\Windows"))
            .unwrap();
        let users_pidl = desktop
            .parse_display_name_to_id_list(HWND::default(), w!(r"C:\Users"))
            .unwrap();
        let (windows_pidl, users_pidl) = (&windows_pidl, &users_pidl);

        let result = desktop
            .compare_ids(Default::default(), windows_pidl, users_pidl)
            .unwrap();
        assert_eq!(result, cmp::Ordering::Greater);

        let result = desktop
            .compare_ids(CompareIDs::CANONICAL_ONLY, windows_pidl, users_pidl)
            .unwrap();
        assert_eq!(result, cmp::Ordering::Greater);
    }

    #[test]
    fn compare_ids_equal() {
        let desktop = IShellFolder::from_desktop().unwrap();

        let pidl1 = desktop
            .parse_display_name_to_id_list(HWND::default(), w!(r"C:\Windows"))
            .unwrap();
        let pidl2 = desktop
            .parse_display_name_to_id_list(HWND::default(), w!(r"C:\Windows"))
            .unwrap();
        let (pidl1, pidl2) = (&pidl1, &pidl2);

        let result = desktop
            .compare_ids(Default::default(), pidl1, pidl2)
            .unwrap();
        assert_eq!(result, cmp::Ordering::Equal);
    }

    #[test]
    fn compare_ids_err() {
        // CompareIDs with invalid PIDs should fail - just verify it compiles and returns Err
        let desktop = IShellFolder::from_desktop().unwrap();
        let pidl1 = RelativeIDList(Default::default());
        let pidl2 = RelativeIDList(Default::default());
        let (pidl1, pidl2) = (&pidl1, &pidl2);
        let result = desktop.compare_ids(Default::default(), pidl1, pidl2);
        assert!(result.is_err());
    }

    #[test]
    fn get_display_name_of() {
        let c = IShellFolder::from_path_w(HWND::default(), w!(r"C:\")).unwrap();

        let windows_pidl = c
            .parse_display_name_to_id_list(HWND::default(), w!(r"Windows"))
            .unwrap();

        // Test with SHGDNF::SHGDN_NORMAL (0)
        let result = c.get_display_name_of(windows_pidl.to_child_ref(), SHGDNF(0));
        assert_eq!(result.unwrap().to_string(), "Windows");

        let result = c.get_path_of(windows_pidl.to_child_ref());
        assert_eq!(result.unwrap().to_string(), r"C:\Windows");
    }
}
