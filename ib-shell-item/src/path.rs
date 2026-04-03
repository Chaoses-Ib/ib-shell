use std::path::{Path, PathBuf};

use widestring::U16CStr;
use windows::{Win32::UI::Shell::IShellItem, core::Result};

use crate::{ShellItem, ShellItemDisplayName, id_list::AbsoluteIDList};

/// A file system path or Windows Shell item ID list (PIDL).
#[derive(Clone)]
pub enum ShellPath<'a> {
    Path(PathBuf),
    // `(*const ITEMIDLIST, PhantomData<&'a ()>)`
    IdList(&'a AbsoluteIDList),
}

impl From<PathBuf> for ShellPath<'static> {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

impl From<&Path> for ShellPath<'static> {
    fn from(value: &Path) -> Self {
        Self::Path(value.into())
    }
}

impl<'a> ShellPath<'a> {
    pub fn from_path_or_id_list(path: *const u16, id_list: &'a AbsoluteIDList) -> Option<Self> {
        match (path.is_null(), id_list.0.is_null()) {
            (true, true) => None,
            // Prefer lpFile
            (false, _) => {
                let file = unsafe { U16CStr::from_ptr_str(path) };

                Self::Path(file.to_os_string().into()).into()
            }
            (true, false) => Self::IdList(id_list).into(),
        }
    }

    pub fn to_file_path(&self) -> Result<PathBuf> {
        Ok(match self {
            ShellPath::Path(path) => path.clone(),
            ShellPath::IdList(id_list) => {
                let item = IShellItem::from_id_list(id_list)?;
                let name = item.get_display_name(ShellItemDisplayName::FileSystemPath)?;
                name.to_os_string().into()
            }
        })
    }
}

impl<'a> std::fmt::Debug for ShellPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(arg0) => f.debug_tuple("Path").field(arg0).finish(),
            Self::IdList(arg0) => {
                let mut f = f.debug_tuple("IdList");
                let f = f.field(arg0);
                if let Ok(path) = self.to_file_path() {
                    f.field(&path);
                }
                f.finish()
            }
        }
    }
}
