use std::cmp;

use bon::Builder;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM},
        UI::Shell::{
            Common::ITEMIDLIST, IShellFolder, SHCIDS_ALLFIELDS, SHCIDS_CANONICALONLY,
            SHGetDesktopFolder,
        },
    },
    core::{PCWSTR, Result},
};

use crate::id_list::RelativeIDList;

/// [IShellFolder::CompareIDs (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-compareids#parameters)
#[derive(Debug, Clone, Copy, Default, Builder)]
pub struct CompareIDs {
    /// - [`property::column::FSColumn`](crate::property::column::FSColumn)
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
    fn from_id_list(pidl: RelativeIDList) -> Result<IShellFolder> {
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
    fn from_path(hwnd: HWND, path: PCWSTR) -> Result<IShellFolder> {
        let desktop = Self::from_desktop()?;
        let pidl = desktop.parse_display_name_to_id_list(hwnd, path)?;
        unsafe { desktop.BindToObject(pidl.0, None) }
    }

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
    fn compare_ids(
        &self,
        param: CompareIDs,
        pidl1: RelativeIDList,
        pidl2: RelativeIDList,
    ) -> Result<cmp::Ordering>;
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
        pidl1: RelativeIDList,
        pidl2: RelativeIDList,
    ) -> Result<cmp::Ordering> {
        let hres = unsafe { self.CompareIDs(param.into(), pidl1.0, pidl2.0) };
        hres.ok()?;
        let code = (hres.0 & 0xFFFF) as i16;
        // dbg!(code, code.cmp(&0));
        Ok(code.cmp(&0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::w;

    use crate::property::column::FSColumn;

    #[test]
    fn from_desktop() {
        let _desktop = IShellFolder::from_desktop().unwrap();
    }

    #[test]
    fn from_path() {
        let _folder = IShellFolder::from_path(HWND::default(), w!(r"C:\Windows")).unwrap();
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
        let c = IShellFolder::from_path(HWND::default(), w!(r"C:\")).unwrap();

        let windows_pidl = c
            .parse_display_name_to_id_list(HWND::default(), w!(r"Windows"))
            .unwrap();
        let users_pidl = c
            .parse_display_name_to_id_list(HWND::default(), w!(r"Users"))
            .unwrap();
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
        let windows = IShellFolder::from_path(HWND::default(), w!(r"C:\Windows")).unwrap();

        let explorer_pidl = windows
            .parse_display_name_to_id_list(HWND::default(), w!(r"explorer.exe"))
            .unwrap();
        let notepad_pidl = windows
            .parse_display_name_to_id_list(HWND::default(), w!(r"notepad.exe"))
            .unwrap();

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

        let result = desktop
            .compare_ids(Default::default(), pidl1, pidl2)
            .unwrap();
        assert_eq!(result, cmp::Ordering::Equal);
    }

    #[test]
    fn compare_ids_err() {
        // CompareIDs with invalid PIDs should fail - just verify it compiles and returns Err
        let desktop = IShellFolder::from_desktop().unwrap();
        let pidl1 = RelativeIDList(std::ptr::null());
        let pidl2 = RelativeIDList(std::ptr::null());
        let result = desktop.compare_ids(Default::default(), pidl1, pidl2);
        assert!(result.is_err());
    }
}
