use std::{cell::SyncUnsafeCell, cmp, ffi::c_void, path::Path};

use bon::Builder;
use ib_hook::inline::InlineHook;
use serde::{Deserialize, Serialize};
use tracing::debug;
use windows::{
    Win32::{
        Foundation::LPARAM,
        UI::Shell::{Common::ITEMIDLIST, IShellFolder},
    },
    core::{HRESULT, Interface},
};

use crate::{
    folder::{CompareIDs as CompareIDsParam, ShellFolder},
    id_list::ChildIDRef,
    prop::column,
};

#[derive(Default, Serialize, Deserialize, Clone, Builder, Debug)]
pub struct FolderHookConfig {
    #[cfg(feature = "everything")]
    #[builder(default)]
    compare_size_from_everything: bool,
}

type CompareIDs =
    unsafe extern "system" fn(*mut c_void, LPARAM, *const ITEMIDLIST, *const ITEMIDLIST) -> HRESULT;

pub(crate) struct FolderHook {
    compare_ids: Option<InlineHook<CompareIDs>>,
}

impl FolderHook {
    pub fn new(config: FolderHookConfig) -> anyhow::Result<Self> {
        let folder = IShellFolder::from_fs_any(Default::default())?;

        let compare_ids = if config.compare_size_from_everything {
            let compare_ids = folder.vtable().CompareIDs;
            InlineHook::new_enabled(compare_ids, compare_ids_detour).ok()
        } else {
            None
        };

        Ok(Self { compare_ids })
    }
}

pub(crate) static HOOK: SyncUnsafeCell<Option<FolderHook>> = SyncUnsafeCell::new(None);

pub fn apply(config: Option<FolderHookConfig>) -> anyhow::Result<()> {
    let hook = unsafe { &mut *HOOK.get() };
    *hook = match config {
        Some(config) => Some(FolderHook::new(config)?),
        None => None,
    };
    Ok(())
}

unsafe extern "system" fn compare_ids_detour(
    this: *mut c_void,
    lparam: LPARAM,
    pidl1: *const ITEMIDLIST,
    pidl2: *const ITEMIDLIST,
) -> HRESULT {
    let hook = unsafe { &*HOOK.get() }.as_ref().unwrap();
    let real =
        || unsafe { hook.compare_ids.as_ref().unwrap().trampoline()(this, lparam, pidl1, pidl2) };

    let param = CompareIDsParam::from(lparam);
    /*
    debug!(?this, ?param, ?pidl1, ?pidl2);
    debug!(folder = ?{
        let folder = unsafe { IShellFolder::from_raw_borrowed(&this) }.unwrap();
        let pidl1 = unsafe { ChildIDRef::from_raw(pidl1) };
        let pidl2 = unsafe { ChildIDRef::from_raw(pidl2) };
        let path1 = folder.get_path_of(pidl1).ok();
        let path2 = folder.get_path_of(pidl2).ok();
        (
            path1,
            path2,
        )
    });
    debug!(
        ?param,
        folder = ?{
            let folder = unsafe { IShellFolder::from_raw_borrowed(&this) }.unwrap();
            let pidl1 = unsafe { ChildIDRef::from_raw(pidl1) };
            let pidl2 = unsafe { ChildIDRef::from_raw(pidl2) };
            let path1 = folder.get_path_of(pidl1).ok();
            let path2 = folder.get_path_of(pidl2).ok();
            (
                folder.is_child_folder(pidl1),
                folder.is_child_folder(pidl2),
                path1,
                path2,
            )
        }
    );
    */

    /*
    static LAST_PARAM: SyncUnsafeCell<u16> = SyncUnsafeCell::new(0);
    static LAST_PIDL1: SyncUnsafeCell<usize> = SyncUnsafeCell::new(0);
    static LAST_PIDL2: SyncUnsafeCell<usize> = SyncUnsafeCell::new(0);
    */

    // 怎么知道现在排序的列和顺序？
    // 列用时间勉强可以
    #[cfg(feature = "everything")]
    if param.column == column::FSColumn::Size as u16
    /*
        || (param.column == column::FSColumn::ItemNameDisplay as u16
            && unsafe { *LAST_PARAM.get() } == column::FSColumn::Size as u16
            && unsafe { *LAST_PIDL1.get() } == pidl1 as usize
            && unsafe { *LAST_PIDL2.get() } == pidl2 as usize)
    */
    {
        /*
        debug!(?this, ?param, ?pidl1, ?pidl2, "Size");
        */
        let folder = unsafe { IShellFolder::from_raw_borrowed(&this) }.unwrap();
        let pidl1 = unsafe { ChildIDRef::from_raw(pidl1) };
        let pidl2 = unsafe { ChildIDRef::from_raw(pidl2) };

        // is_child_has_subfolder() may cause serious lag
        let is_folder1 = folder.is_child_fs_folder(pidl1);
        let is_folder2 = folder.is_child_fs_folder(pidl2);

        /*
        debug!(?param, path1 = ?folder.get_path_of(pidl1).ok(), path2 = ?folder.get_path_of(pidl2).ok(), is_folder1, is_folder2, "Size");
        */
        if is_folder1 && is_folder2 {
            let compare = || {
                let path1 = folder.get_path_of(pidl1).ok()?.to_string();
                let path2 = folder.get_path_of(pidl2).ok()?.to_string();
                #[cfg(debug_assertions)]
                if path1 == path2 {
                    // Yes, this may happen, and pidl1 != pidl2, on at least Windows 11 24H2
                    debug!("same path");
                    return Some(cmp::Ordering::Equal);
                }
                let path1 = Path::new(&path1);
                let path2 = Path::new(&path2);

                let size1 = everything_ipc::folder::size::get_folder_size(path1)
                    .eager_get_links(true)
                    .call()
                    .ok()?;
                /*
                if size1 == 0 {
                    return Some(cmp::Ordering::Less);
                }
                */
                let size2 = everything_ipc::folder::size::get_folder_size(path2)
                    .eager_get_links(true)
                    .call()
                    .ok()?;

                let r = size1.cmp(&size2);
                debug!(?param, ?path1, ?path2, ?r);
                Some(r)
            };
            if let Some(r) = compare() {
                /*
                if (r as i32) < 0 {
                    unsafe { *LAST_PARAM.get() = param.column };
                    unsafe { *LAST_PIDL1.get() = pidl1.0 as usize };
                    unsafe { *LAST_PIDL2.get() = pidl2.0 as usize };
                }
                */
                return CompareIDsParam::to_result(Some(r));
            }
        }
    }

    real()
}
