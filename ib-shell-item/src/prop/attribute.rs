use bitflags::bitflags;
use windows::Win32::System::SystemServices::SFGAO_FLAGS;

/// These flags represent attributes that can be retrieved on an item (file or folder) or set of items.
///
/// They are used with the [`ShellFolder`](crate::folder::ShellFolder) and [`ShellItem`](crate::item::ShellItem) APIs,
/// most notably [`ShellFolder::get_attributes_of`](crate::folder::ShellFolder::get_attributes_of)
/// and [`ShellItem::get_attributes`](crate::item::ShellItem::get_attributes).
///
/// [SFGAO (Shobjidl.h)](https://learn.microsoft.com/en-us/windows/win32/shell/sfgao)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemAttributes(u32);

impl From<SFGAO_FLAGS> for ItemAttributes {
    fn from(value: SFGAO_FLAGS) -> Self {
        Self(value.0)
    }
}

impl Into<SFGAO_FLAGS> for ItemAttributes {
    fn into(self) -> SFGAO_FLAGS {
        SFGAO_FLAGS(self.0)
    }
}

bitflags! {
    impl ItemAttributes: u32 {
        /// The specified items can be copied.
        const CanCopy = 0x00000001;

        /// The specified items can be moved.
        const CanMove = 0x00000002;

        /// Shortcuts can be created for the specified items.
        ///
        /// This attribute has the same value as `DROPEFFECT_LINK`.
        /// If a namespace extension returns this attribute, a `Create Shortcut`
        /// entry with a default handler is added to the shortcut menu that is
        /// displayed during drag-and-drop operations. The extension can also
        /// implement its own handler for the `link` verb in place of the default.
        /// If the extension does so, it is responsible for creating the shortcut.
        ///
        /// A `Create Shortcut` item is also added to the Windows Explorer `File`
        /// menu and to normal shortcut menus. If the item is selected, your
        /// application's `IContextMenu::InvokeCommand` method is invoked with
        /// the `lpVerb` member of the `CMINVOKECOMMANDINFO` structure set to
        /// `link`. Your application is responsible for creating the link.
        const CanLink = 0x00000004;

        /// The specified items can be bound to an `IStorage` object through
        /// `IShellFolder::BindToObject`. For more information about namespace
        /// manipulation capabilities, see `IStorage`.
        const Storage = 0x00000008;

        /// The specified items can be renamed.
        ///
        /// Note that this value is essentially a suggestion; not all namespace
        /// clients allow items to be renamed. However, those that do must have
        /// this attribute set.
        const CanRename = 0x00000010;

        /// The specified items can be deleted.
        const CanDelete = 0x00000020;

        /// The specified items have property sheets.
        const HasPropSheet = 0x00000040;

        /// The specified items are drop targets.
        const DropTarget = 0x00000100;

        /// This flag is a mask for the capability attributes: [`ItemAttributes::CanCopy`],
        /// [`ItemAttributes::CanMove`], [`ItemAttributes::CanLink`], [`ItemAttributes::CanRename`], [`ItemAttributes::CanDelete`],
        /// [`ItemAttributes::HasPropSheet`], and [`ItemAttributes::DropTarget`].
        ///
        /// Callers normally do not use this value.
        const CapabilityMask = 0x00000177;

        /// Windows 7 and later. The specified items are system items.
        const System = 0x00001000;

        /// The specified items are encrypted and might require special presentation.
        const Encrypted = 0x00002000;

        /// Accessing the item (through `IStream` or other storage interfaces) is
        /// expected to be a slow operation.
        /// Applications should avoid accessing items flagged with [`ItemAttributes::IsSlow`].
        ///
        /// Note: Opening a stream for an item is generally a slow operation at
        /// all times. [`ItemAttributes::IsSlow`] indicates that it is expected to be
        /// especially slow, for example in the case of slow network connections
        /// or offline (`FILE_ATTRIBUTE_OFFLINE`) files. However, querying
        /// [`ItemAttributes::IsSlow`] is itself a slow operation. Applications should query
        /// [`ItemAttributes::IsSlow`] only on a background thread. An alternate method,
        /// such as retrieving the `PKEY_FileAttributes` property and testing for
        /// `FILE_ATTRIBUTE_OFFLINE`, could be used in place of a method call
        /// that involves [`ItemAttributes::IsSlow`].
        const IsSlow = 0x00004000;

        /// The specified items are shown as dimmed and unavailable to the user.
        const Ghosted = 0x00008000;

        /// The specified items are shortcuts.
        const Link = 0x00010000;

        /// The specified objects are shared.
        const Share = 0x00020000;

        /// The specified items are read-only.
        ///
        /// In the case of folders, this means that new items cannot be created
        /// in those folders. This should not be confused with the behavior
        /// specified by the `FILE_ATTRIBUTE_READONLY` flag retrieved by
        /// `IColumnProvider::GetItemData` in a `SHCOLUMNDATA` structure.
        /// `FILE_ATTRIBUTE_READONLY` has no meaning for Win32 file system folders.
        const ReadOnly = 0x00040000;

        /// The item is hidden and should not be displayed unless the `Show hidden
        /// files and folders` option is enabled in `Folder Settings`.
        const Hidden = 0x00080000;

        /// Do not use.
        const DisplayAttrMask = 0x000FC000;

        /// The items are nonenumerated items and should be hidden.
        ///
        /// They are not returned through an enumerator such as that created by
        /// the `IShellFolder::EnumObjects` method.
        const NonEnumerated = 0x00100000;

        /// The items contain new content, as defined by the particular application.
        const NewContent = 0x00200000;

        /// Not supported.
        const CanMoniker = 0x00400000;

        /// Not supported.
        const HasStorage = 0x00800000;

        /// Indicates that the item has a stream associated with it.
        ///
        /// That stream can be accessed through a call to
        /// `IShellFolder::BindToObject` or `IShellItem::BindToHandler` with
        /// `IID_IStream` in the `riid` parameter.
        const Stream = 0x01000000;

        /// Children of this item are accessible through `IStream` or `IStorage`.
        ///
        /// Those children are flagged with [`ItemAttributes::Storage`] or [`ItemAttributes::Stream`].
        const StorageAncestor = 0x02000000;

        /// When specified as input, [`ItemAttributes::Validate`] instructs the folder to
        /// validate that the items contained in a folder or Shell item array
        /// exist.
        ///
        /// If one or more of those items do not exist,
        /// `IShellFolder::GetAttributesOf` and `IShellItemArray::GetAttributes`
        /// return a failure code. This flag is never returned as an `out` value.
        /// When used with the file system folder, [`ItemAttributes::Validate`] instructs the
        /// folder to discard cached properties retrieved by clients of
        /// `IShellFolder2::GetDetailsEx` that might have accumulated for the
        /// specified items.
        const Validate = 0x04000000;

        /// The specified items are on removable media or are themselves removable devices.
        const Removable = 0x08000000;

        /// The specified items are compressed.
        const Compressed = 0x10000000;

        /// The specified items can be hosted inside a web browser or Windows Explorer frame.
        const Browsable = 0x20000000;

        /// The specified folders are either file system folders or contain at
        /// least one descendant (child, grandchild, or later) that is a file
        /// system ([`ItemAttributes::FileSystem`]) folder.
        const FileSysAncestor = 0x40000000;

        /// The specified items are folders.
        ///
        /// Some items can be flagged with both [`ItemAttributes::Stream`] and [`ItemAttributes::Folder`],
        /// such as a compressed file with a `.zip` file name extension. Some
        /// applications might include this flag when testing for items that are
        /// both files and containers.
        const Folder = 0x80000000;

        /// The specified folders or files are part of the file system (that is,
        /// they are files, directories, or root directories).
        ///
        /// The parsed names of the items can be assumed to be valid Win32 file
        /// system paths. These paths can be either UNC or drive-letter based.
        const FileSystem = 0x80000000;

        /// This flag is a mask for the storage capability attributes:
        /// [`ItemAttributes::Storage`], [`ItemAttributes::Link`], [`ItemAttributes::ReadOnly`], [`ItemAttributes::Stream`],
        /// [`ItemAttributes::StorageAncestor`], [`ItemAttributes::FileSysAncestor`], [`ItemAttributes::Folder`],
        /// and [`ItemAttributes::FileSystem`].
        ///
        /// Callers normally do not use this value.
        const StorageCapMask = 0x70C50008;

        /// The specified folders have subfolders.
        ///
        /// The [`ItemAttributes::HasSubFolder`] attribute is only advisory and might be
        /// returned by Shell folder implementations even if they do not contain
        /// subfolders. Note, however, that the converse—failing to return
        /// [`ItemAttributes::HasSubFolder`]—definitively states that the folder objects do
        /// not have subfolders. Returning [`ItemAttributes::HasSubFolder`] is recommended
        /// whenever a significant amount of time is required to determine whether
        /// any subfolders exist. For example, the Shell always returns
        /// [`ItemAttributes::HasSubFolder`] when a folder is located on a network drive.
        const HasSubFolder = 0x80000000;

        /// This flag is a mask for content attributes, at present only
        /// [`ItemAttributes::HasSubFolder`].
        ///
        /// Callers normally do not use this value.
        const ContentMask = 0x80000000;

        /// Mask used by the `PKEY_SFGAOFlags` property to determine attributes
        /// that are considered to cause slow calculations or lack context:
        /// [`ItemAttributes::IsSlow`], [`ItemAttributes::ReadOnly`], [`ItemAttributes::HasSubFolder`], and
        /// [`ItemAttributes::Validate`].
        ///
        /// Callers normally do not use this value.
        const PkeySfgaoMask = 0x81044000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use windows::{
        Win32::{
            Foundation::HWND,
            UI::Shell::{IShellFolder, IShellItem},
        },
        core::w,
    };

    use crate::{ShellItem, folder::ShellFolder, init};

    #[test]
    fn item() {
        _ = init();

        let windows = IShellItem::from_path_w(w!(r"C:\Windows")).unwrap();
        assert_eq!(
            windows.get_attributes(ItemAttributes::Folder),
            Ok(ItemAttributes::Folder)
        );
        assert!(windows.is_folder());

        let explorer = IShellItem::from_path_w(w!(r"C:\Windows\explorer.exe")).unwrap();
        assert_eq!(
            explorer.get_attributes(ItemAttributes::Folder),
            Ok(ItemAttributes::empty())
        );
        assert!(!explorer.is_folder());
    }

    #[allow(deprecated)]
    #[test]
    fn folder() {
        // Get the C:\Windows folder
        let windows_folder = IShellFolder::from_path_w(HWND::default(), w!(r"C:\Windows")).unwrap();

        // Get child PIDs relative to the Windows folder
        // Use "system32" as a subfolder and "notepad.exe" as a file
        let system32_pidl = windows_folder
            .parse_display_name_to_id_list(HWND::default(), w!(r"system32"))
            .unwrap()
            .into_child();
        let notepad_pidl = windows_folder
            .parse_display_name_to_id_list(HWND::default(), w!(r"notepad.exe"))
            .unwrap()
            .into_child();

        // Query folder attribute for system32 folder - should have Folder bit
        let system32_attrs = windows_folder
            .get_attributes_of(&[system32_pidl.to_ref()], ItemAttributes::Folder)
            .unwrap();
        // system32 is a folder, so it must have the Folder bit set
        assert!(system32_attrs.contains(ItemAttributes::Folder));

        // Query folder attribute for notepad.exe - should not have Folder bit
        let notepad_attrs = windows_folder
            .get_attributes_of(&[notepad_pidl.to_ref()], ItemAttributes::Folder)
            .unwrap();
        assert!(!notepad_attrs.contains(ItemAttributes::Folder));

        // Query multiple items - folder and file - Folder bit should not be common
        let children = &[system32_pidl.to_ref(), notepad_pidl.to_ref()];
        let multi_attrs = windows_folder
            .get_attributes_of(children, ItemAttributes::Folder)
            .unwrap();
        // Since notepad.exe is not a folder, the common attributes should not include Folder
        assert!(!multi_attrs.contains(ItemAttributes::Folder));
        assert!(!windows_folder.are_children_folders(children));

        // Not supported
        let children = &[system32_pidl.to_ref(), system32_pidl.to_ref()];
        let multi_attrs = windows_folder
            .get_attributes_of(children, ItemAttributes::Folder)
            .unwrap();
        assert!(!multi_attrs.contains(ItemAttributes::Folder));
        assert!(!windows_folder.are_children_folders(children));

        // Not supported
        let system_pidl = windows_folder
            .parse_display_name_to_id_list(HWND::default(), w!(r"System"))
            .unwrap()
            .into_child();
        let children = &[system32_pidl.to_ref(), system_pidl.to_ref()];
        let multi_attrs = windows_folder
            .get_attributes_of(children, ItemAttributes::Folder)
            .unwrap();
        assert!(!multi_attrs.contains(ItemAttributes::Folder));
        assert!(!windows_folder.are_children_folders(children));
    }
}
