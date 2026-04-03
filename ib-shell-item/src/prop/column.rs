use derive_more::TryFrom;
use num_enum::IntoPrimitive;
use windows::Win32::{Foundation::PROPERTYKEY, Storage::EnhancedStorage::*};

/**
File system item columns.

Note that columns starting from [`FSColumn::DateCreated`] are off by default
(don't have [`SHCOLSTATE_ONBYDEFAULT`] set);
undocumented and possible (although unlikely) to change.

Columns until [`FSColumn::OfflineStatus`] (included) are available from Windows XP SP1.
*/
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFrom, IntoPrimitive)]
#[try_from(repr)]
#[repr(u16)]
pub enum FSColumn {
    ItemNameDisplay = 0,
    Size = 1,
    ItemType = 2,
    DateModified = 3,
    DateCreated = 4,
    DateAccessed = 5,
    FileAttributes = 6,
    OfflineStatus = 7,
    OfflineAvailability = 8,
    PerceivedType = 9,
    FileOwner = 10,
    Kind = 11,
    Photo_DateTaken = 12,
    Music_Artist = 13,
    Music_AlbumTitle = 14,
    Media_Year = 15,
    Music_Genre = 16,
    Music_Conductor = 17,
    Keywords = 18,
    Rating = 19,
    Author = 20,
    Title = 21,
    Subject = 22,
    Category = 23,
    Comment = 24,
    Copyright = 25,
    Music_TrackNumber = 26,
    Media_Duration = 27,
    Audio_EncodingBitrate = 28,
    DRM_IsProtected = 29,
    Photo_CameraModel = 30,
    Image_Dimensions = 31,
    Photo_CameraManufacturer = 32,
    Company = 33,
    FileDescription = 34,
}

impl FSColumn {
    pub fn key(self) -> PROPERTYKEY {
        use FSColumn::*;
        match self {
            ItemNameDisplay => PKEY_ItemNameDisplay,
            Size => PKEY_Size,
            ItemType => PKEY_ItemType,
            DateModified => PKEY_DateModified,
            DateCreated => PKEY_DateCreated,
            DateAccessed => PKEY_DateAccessed,
            FileAttributes => PKEY_FileAttributes,
            OfflineStatus => PKEY_OfflineStatus,
            OfflineAvailability => PKEY_OfflineAvailability,
            PerceivedType => PKEY_PerceivedType,
            FileOwner => PKEY_FileOwner,
            Kind => PKEY_Kind,
            Photo_DateTaken => PKEY_Photo_DateTaken,
            Music_Artist => PKEY_Music_Artist,
            Music_AlbumTitle => PKEY_Music_AlbumTitle,
            Media_Year => PKEY_Media_Year,
            Music_Genre => PKEY_Music_Genre,
            Music_Conductor => PKEY_Music_Conductor,
            Keywords => PKEY_Keywords,
            Rating => PKEY_Rating,
            Author => PKEY_Author,
            Title => PKEY_Title,
            Subject => PKEY_Subject,
            Category => PKEY_Category,
            Comment => PKEY_Comment,
            Copyright => PKEY_Copyright,
            Music_TrackNumber => PKEY_Music_TrackNumber,
            Media_Duration => PKEY_Media_Duration,
            Audio_EncodingBitrate => PKEY_Audio_EncodingBitrate,
            DRM_IsProtected => PKEY_DRM_IsProtected,
            Photo_CameraModel => PKEY_Photo_CameraModel,
            Image_Dimensions => PKEY_Image_Dimensions,
            Photo_CameraManufacturer => PKEY_Photo_CameraManufacturer,
            Company => PKEY_Company,
            FileDescription => PKEY_FileDescription,
        }
    }
}
