use windows::{
    Win32::UI::Shell::{
        Common::ITEMIDLIST,
        PropertiesSystem::{GETPROPERTYSTOREFLAGS, IPropertyStore},
        SHCreateItemFromIDList, SHCreateItemFromParsingName,
    },
    core::{PCWSTR, Result},
};

pub use windows::Win32::UI::Shell::IShellItem2;

pub trait ShellItem2 {
    /// [SHCreateItemFromParsingName function (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromparsingname)
    ///
    /// Although not documented, this requires `CoInitialize()`.
    fn from_path_w(path: PCWSTR) -> Result<IShellItem2>;

    /// [SHCreateItemFromIDList function (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromidlist)
    #[doc(alias = "from_pidl")]
    fn from_id_list(id_list: *const ITEMIDLIST) -> Result<IShellItem2>;

    /// [IShellItem2::GetPropertyStore (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitem2-getpropertystore)
    fn get_property_store(&self, flags: GETPROPERTYSTOREFLAGS) -> Result<IPropertyStore>;
}

impl ShellItem2 for IShellItem2 {
    fn from_path_w(path: PCWSTR) -> Result<IShellItem2> {
        unsafe { SHCreateItemFromParsingName::<_, _, IShellItem2>(path, None) }
    }

    fn from_id_list(id_list: *const ITEMIDLIST) -> Result<IShellItem2> {
        unsafe { SHCreateItemFromIDList::<IShellItem2>(id_list) }
    }

    fn get_property_store(&self, flags: GETPROPERTYSTOREFLAGS) -> Result<IPropertyStore> {
        unsafe { self.GetPropertyStore(flags) }
    }
}
