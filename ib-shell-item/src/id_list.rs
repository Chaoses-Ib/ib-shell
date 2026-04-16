/*!
Shell item ID list (path).
*/
use std::{marker::PhantomData, mem, ops::Deref, os::raw::c_void};

use derive_more::From;
use windows::{
    Win32::{
        System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree},
        UI::Shell::{Common::ITEMIDLIST, IPersistIDList, IShellItem, SHGetIDListFromObject},
    },
    core::{IUnknown, Interface, Param, Result},
};

use super::item::ShellItem;

#[derive(Debug, Clone, From)]
pub struct ChildID(pub *mut ITEMIDLIST);

impl From<RelativeIDList> for ChildID {
    fn from(value: RelativeIDList) -> Self {
        let id = Self(value.0);
        mem::forget(value);
        id
    }
}

impl ChildID {
    pub fn to_ref(&self) -> ChildIDRef<'_> {
        self.into()
    }
}

impl Drop for ChildID {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.cast())) }
    }
}

/// This has the same memory representation as [`ChildID`],
/// but represents a borrowed ID list pointer.
///
/// You should only use this in C arrays/structs, otherwise `&ChildID` is enough and more idiomatic.
#[derive(Debug, Clone, Copy)]
pub struct ChildIDRef<'a>(pub *const ITEMIDLIST, PhantomData<&'a ()>);

impl<'a> ChildIDRef<'a> {
    /// Creates a `ChildIDRef` from a raw pointer.
    pub unsafe fn from_raw(ptr: *const ITEMIDLIST) -> Self {
        Self(ptr, PhantomData)
    }
}

impl<'a> From<&'a ChildID> for ChildIDRef<'a> {
    fn from(id: &'a ChildID) -> Self {
        Self(id.0, PhantomData)
    }
}

impl<'a> Deref for ChildIDRef<'a> {
    type Target = ChildID;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self) }
    }
}

#[derive(Debug, Clone, From)]
pub struct RelativeIDList(pub *mut ITEMIDLIST);

impl Drop for RelativeIDList {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.cast())) }
    }
}

/// This has the same memory representation as [`RelativeIDList`],
/// but represents a borrowed ID list pointer.
///
/// You should only use this in C arrays/structs, otherwise `&RelativeIDList` is enough and more idiomatic.
#[derive(Debug, Clone, Copy)]
pub struct RelativeIDListRef<'a>(pub *const ITEMIDLIST, PhantomData<&'a ()>);

impl<'a> RelativeIDListRef<'a> {
    /// Creates a `RelativeIDListRef` from a raw pointer.
    pub unsafe fn from_raw(ptr: *const ITEMIDLIST) -> Self {
        Self(ptr, PhantomData)
    }
}

impl<'a> From<&'a RelativeIDList> for RelativeIDListRef<'a> {
    fn from(id: &'a RelativeIDList) -> Self {
        Self(id.0, PhantomData)
    }
}

impl<'a> Deref for RelativeIDListRef<'a> {
    type Target = RelativeIDList;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self) }
    }
}

impl RelativeIDList {
    /// Converts this [`RelativeIDList`] to a [`ChildID`], transferring ownership.
    /// The underlying PIDL should be a valid child item ID.
    ///
    /// ## Safety
    /// This is safe as most APIs either check or don't care.
    pub fn into_child(self) -> ChildID {
        self.into()
    }

    /// Converts this [`RelativeIDList`] to a [`ChildIDRef`] without transferring ownership.
    ///
    /// ## Safety
    /// This is safe as most APIs either check or don't care.
    pub fn to_child_ref(&self) -> ChildIDRef<'_> {
        ChildIDRef(self.0, PhantomData)
    }
}

#[derive(Debug, Clone, From)]
pub struct AbsoluteIDList(pub *mut ITEMIDLIST);

impl AbsoluteIDList {
    pub fn from_raw_void_ref<'a>(pidl: &'a *mut c_void) -> &'a Self {
        unsafe { mem::transmute(pidl) }
    }

    /// The following classes are not supported:
    /// - [`PropertyStore`](crate::prop::store::PropertyStore): `CFSFolderPropertyStore`
    ///
    /// [SHGetIDListFromObject function (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shgetidlistfromobject)
    pub fn from_object(unk: impl Param<IUnknown>) -> Result<Self> {
        unsafe { SHGetIDListFromObject(unk) }.map(AbsoluteIDList)
    }
}

impl Drop for AbsoluteIDList {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.cast())) }
    }
}

/// This has the same memory representation as [`AbsoluteIDList`],
/// but represents a borrowed ID list pointer.
///
/// You should only use this in C arrays/structs, otherwise `&AbsoluteIDList` is enough and more idiomatic.
#[derive(Debug, Clone, Copy)]
pub struct AbsoluteIDListRef<'a>(pub *const ITEMIDLIST, PhantomData<&'a ()>);

impl<'a> AbsoluteIDListRef<'a> {
    /// Creates an `AbsoluteIDListRef` from a raw pointer.
    pub unsafe fn from_raw(ptr: *const ITEMIDLIST) -> Self {
        Self(ptr, PhantomData)
    }
}

impl<'a> From<&'a AbsoluteIDList> for AbsoluteIDListRef<'a> {
    fn from(id: &'a AbsoluteIDList) -> Self {
        Self(id.0, PhantomData)
    }
}

impl<'a> Deref for AbsoluteIDListRef<'a> {
    type Target = AbsoluteIDList;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self) }
    }
}

/// [IPersistIDList (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ipersistidlist)
pub trait PersistIDList {
    fn new() -> Result<IPersistIDList>;

    /// [IPersistIDList::GetIDList (shobjidl_core.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ipersistidlist-getidlist)
    ///
    /// ## References
    /// [Simon Mourier's Blog - How to programmatically switch a File Explorer view to thumbnails?](https://www.simonmourier.com/blog/How-to-programmatically-switch-a-File-Explorer-view-to-thumbnails/)
    fn get_id_list(&self) -> Result<AbsoluteIDList>;

    fn to_shell_item(&self) -> Result<IShellItem>;
}

impl PersistIDList for IPersistIDList {
    fn new() -> Result<IPersistIDList> {
        unsafe { CoCreateInstance(&IPersistIDList::IID, None, CLSCTX_ALL) }
    }

    fn get_id_list(&self) -> Result<AbsoluteIDList> {
        unsafe { self.GetIDList() }.map(AbsoluteIDList)
    }

    fn to_shell_item(&self) -> Result<IShellItem> {
        let id_list = self.get_id_list()?;
        let item = IShellItem::from_id_list(&id_list)?;
        // unsafe { CoTaskMemFree(Some(id_list as _)) };
        Ok(item)
    }
}
