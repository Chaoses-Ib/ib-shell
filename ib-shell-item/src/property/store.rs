use windows::{
    Win32::{
        Foundation::PROPERTYKEY,
        Storage::EnhancedStorage::{
            PKEY_ItemPathDisplay, PKEY_ParsingName, PKEY_ParsingPath, PKEY_Size,
        },
        System::Com::StructuredStorage::PROPVARIANT,
        UI::Shell::{IShellItem2, PropertiesSystem::GETPROPERTYSTOREFLAGS},
    },
    core::{PCWSTR, Result},
};

pub use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;

use crate::item2::ShellItem2;

/// [IPropertyStore (propsys.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nn-propsys-ipropertystore)
pub trait PropertyStore {
    /// [SHCreateItemFromParsingName function (shobjidl_core.h)](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-shcreateitemfromparsingname)
    ///
    /// Although not documented, this requires `CoInitialize()`.
    fn from_path_w(path: PCWSTR, flags: GETPROPERTYSTOREFLAGS) -> Result<IPropertyStore>;

    /// [IPropertyStore::GetValue (propsys.h)](https://learn.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-ipropertystore-getvalue)
    ///
    /// ## Returns
    /// [PROPVARIANT (propidlbase.h)](https://learn.microsoft.com/en-us/windows/win32/api/propidlbase/ns-propidlbase-propvariant)
    fn get_value(&self, key: &PROPERTYKEY) -> Result<PROPVARIANT>;

    /// Get the size of the item.
    ///
    /// ## Returns
    /// Of type [`VT_UI8`].
    fn get_size(&self) -> Result<PROPVARIANT>;

    /// Get the size of the item.
    fn get_size_u64(&self) -> Result<u64>;

    /// Get the parsing (non-UI) path of the item.
    ///
    /// [System.ParsingPath](https://learn.microsoft.com/en-us/windows/win32/properties/props-system-parsingpath)
    ///
    /// [`PKEY_ItemPathDisplay`] is UI path, [`PKEY_ItemUrl`] is broken, but [`PKEY_ParsingPath`] seems fixed.
    ///
    /// Ref: [Vista: How to retrieve the pIDL of a localized path?](https://microsoft.public.platformsdk.shell.narkive.com/Lt8tnJRx/vista-how-to-retrieve-the-pidl-of-a-localized-path)
    ///
    /// ## Examples
    /**
    ```
    use ib_shell_item::{
        init,
        property::store::{IPropertyStore, PropertyStore},
    };
    use windows::core::w;
    _ = init();

    let prop_store = IPropertyStore::from_path_w(
        w!(r"C:\Users\Public\Documents\desktop.ini"),
        Default::default(),
    )
    .unwrap();

    let path = prop_store.get_parsing_path().unwrap().to_string();
    assert_eq!(path, r"C:\Users\Public\Documents\desktop.ini");

    let path = prop_store.get_item_path_display().unwrap().to_string();
    assert_eq!(path, r"C:\Users\Public\Public Documents\desktop.ini");
    ```
    */
    fn get_parsing_path(&self) -> Result<PROPVARIANT>;

    /// Get the parsing name of the item (e.g., "explorer.exe").
    fn get_parsing_name(&self) -> Result<PROPVARIANT>;

    /// Get the display (UI) path of the item.
    ///
    /// [System.ItemPathDisplay](https://learn.microsoft.com/en-us/windows/win32/properties/props-system-itempathdisplay)

    /// ## Examples
    /**
    ```
    use ib_shell_item::{
        init,
        property::store::{IPropertyStore, PropertyStore},
    };
    use windows::core::w;
    _ = init();

    let prop_store = IPropertyStore::from_path_w(
        w!(r"C:\Users\Public\Documents\desktop.ini"),
        Default::default(),
    )
    .unwrap();

    let path = prop_store.get_parsing_path().unwrap().to_string();
    assert_eq!(path, r"C:\Users\Public\Documents\desktop.ini");

    let path = prop_store.get_item_path_display().unwrap().to_string();
    assert_eq!(path, r"C:\Users\Public\Public Documents\desktop.ini");
    ```
    */
    fn get_item_path_display(&self) -> Result<PROPVARIANT>;
}

impl PropertyStore for IPropertyStore {
    fn from_path_w(path: PCWSTR, flags: GETPROPERTYSTOREFLAGS) -> Result<IPropertyStore> {
        IShellItem2::from_path_w(path)?.get_property_store(flags)
    }

    fn get_value(&self, key: &PROPERTYKEY) -> Result<PROPVARIANT> {
        unsafe { self.GetValue(key) }
    }

    fn get_size(&self) -> Result<PROPVARIANT> {
        self.get_value(&PKEY_Size)
    }

    fn get_size_u64(&self) -> Result<u64> {
        (&self.get_size()?).try_into()
    }

    fn get_parsing_path(&self) -> Result<PROPVARIANT> {
        self.get_value(&PKEY_ParsingPath)
    }

    fn get_parsing_name(&self) -> Result<PROPVARIANT> {
        self.get_value(&PKEY_ParsingName)
    }

    fn get_item_path_display(&self) -> Result<PROPVARIANT> {
        self.get_value(&PKEY_ItemPathDisplay)
    }
}

#[cfg(test)]
mod tests {
    use windows::{
        Win32::Storage::EnhancedStorage::{
            PKEY_ImageParsingName, PKEY_ItemUrl, PKEY_Link_TargetParsingPath,
        },
        core::w,
    };

    use super::*;

    use crate::init;

    #[test]
    fn get_value() {
        _ = init();

        let prop_store =
            IPropertyStore::from_path_w(w!(r"C:\Windows\explorer.exe"), Default::default())
                .expect("Failed to create property store from path");

        // Verify we can extract the size value
        let size: u64 = prop_store.get_size_u64().unwrap();
        dbg!(size);
        assert!(size > 0);

        let name = prop_store.get_parsing_name().unwrap().to_string();
        assert_eq!(name, "explorer.exe");

        let name = prop_store
            .get_value(&PKEY_ImageParsingName)
            .unwrap()
            .to_string();
        assert_eq!(name, "");

        let path = prop_store.get_parsing_path().unwrap().to_string();
        assert_eq!(path, r"C:\Windows\explorer.exe");

        let path = prop_store.get_item_path_display().unwrap().to_string();
        assert_eq!(path, r"C:\Windows\explorer.exe");

        let url = prop_store.get_value(&PKEY_ItemUrl).unwrap().to_string();
        assert_eq!(url, "");

        let path = prop_store
            .get_value(&PKEY_Link_TargetParsingPath)
            .unwrap()
            .to_string();
        assert_eq!(path, "");
    }

    #[test]
    fn get_parsing_path() {
        _ = init();

        let prop_store = IPropertyStore::from_path_w(
            w!(r"C:\Users\Public\Documents\desktop.ini"),
            Default::default(),
        )
        .unwrap();

        let path = prop_store.get_parsing_path().unwrap().to_string();
        assert_eq!(path, r"C:\Users\Public\Documents\desktop.ini");

        let path = prop_store.get_item_path_display().unwrap().to_string();
        assert_eq!(path, r"C:\Users\Public\Public Documents\desktop.ini");
    }
}
