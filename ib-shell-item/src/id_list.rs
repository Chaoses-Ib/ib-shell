use derive_more::From;
use windows::{
    Win32::{
        System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree},
        UI::Shell::{Common::ITEMIDLIST, IPersistIDList, IShellItem, SHGetIDListFromObject},
    },
    core::{IUnknown, Interface, Param, Result},
};

use super::item::ShellItem;

#[derive(Debug, Clone, Copy, From)]
pub struct ChildID(pub *mut ITEMIDLIST);

#[derive(Debug, Clone, Copy, From)]
pub struct RelativeIDList(pub *mut ITEMIDLIST);

#[derive(Debug, Clone, Copy, From)]
pub struct AbsoluteIDList(pub *mut ITEMIDLIST);

impl AbsoluteIDList {
    /// The following classes are not supported:
    /// - [`PropertyStore`](crate::property::store::PropertyStore): `CFSFolderPropertyStore`
    ///
    /// [SHGetIDListFromObject function (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shgetidlistfromobject)
    pub fn from_object(unk: impl Param<IUnknown>) -> Result<Self> {
        unsafe { SHGetIDListFromObject(unk) }.map(AbsoluteIDList)
    }
}

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
