use windows::{
    Win32::{
        System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree},
        UI::Shell::{Common::ITEMIDLIST, IPersistIDList, IShellItem},
    },
    core::{Interface, Result},
};

use super::item::ShellItem;

/// [IPersistIDList (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ipersistidlist)
pub trait PersistIDList {
    fn new() -> Result<IPersistIDList>;

    /// [IPersistIDList::GetIDList (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ipersistidlist-getidlist)
    fn get_id_list(&self) -> Result<*mut ITEMIDLIST>;

    fn to_shell_item(&self) -> Result<IShellItem>;
}

impl PersistIDList for IPersistIDList {
    fn new() -> Result<IPersistIDList> {
        unsafe { CoCreateInstance(&IPersistIDList::IID, None, CLSCTX_ALL) }
    }

    fn get_id_list(&self) -> Result<*mut ITEMIDLIST> {
        unsafe { self.GetIDList() }
    }

    fn to_shell_item(&self) -> Result<IShellItem> {
        let id_list = self.get_id_list()?;
        let item = IShellItem::from_id_list(id_list)?;
        unsafe { CoTaskMemFree(Some(id_list as _)) };
        Ok(item)
    }
}
