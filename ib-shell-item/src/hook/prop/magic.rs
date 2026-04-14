use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;

/// "ibsh"
pub const VALUE_MAGIC: u32 = 0x68736269;

pub fn ui8_write_u64(pv: *mut PROPVARIANT, v: u64) {
    unsafe {
        let p = &mut (*(*pv).Anonymous.Anonymous).Anonymous as *mut _ as *mut u8;
        p.add(8).cast::<u32>().write(VALUE_MAGIC);
        p.add(12).cast::<u32>().write(v as u32);
        (*(*pv).Anonymous.Anonymous).wReserved2 = (v >> 32) as u16;
        (*(*pv).Anonymous.Anonymous).wReserved3 = (v >> 48) as u16;
    }
}

pub fn ui8_read_u64(pv: *const PROPVARIANT) -> Option<u64> {
    unsafe {
        let p = &(*(*pv).Anonymous.Anonymous).Anonymous as *const _ as *const u8;
        let tag = p.add(8).cast::<u32>().read();
        if tag == VALUE_MAGIC {
            let a = p.add(12).cast::<u32>().read();
            let b = (*(*pv).Anonymous.Anonymous).wReserved2;
            let c = (*(*pv).Anonymous.Anonymous).wReserved3;
            Some(a as u64 | ((b as u64) << 32) | ((c as u64) << 48))
        } else {
            None
        }
    }
}
