use std::cmp;

use windows::{
    Win32::{Foundation::LPARAM, UI::Shell::SHCIDS_COLUMNMASK},
    core::HRESULT,
};

use crate::folder::CompareIDs;

impl CompareIDs {
    /**
    [MAKE_HRESULT macro (dmerror.h)](https://learn.microsoft.com/en-us/windows/win32/api/dmerror/nf-dmerror-make_hresult)

    ```x86asm
    cmp     dil, 2
    movsx   eax, dil
    movzx   ecx, ax
    mov     eax, -2147483648
    cmovne  eax, ecx
    ret
    ```
    */
    pub fn to_result(order: Option<cmp::Ordering>) -> HRESULT {
        /*
        ```x86asm
        new_result:
        inc     dil
        movzx   eax, dil
        lea     rcx, [rip + .Lswitch.table.new_result]
        mov     eax, dword ptr [rcx + 4*rax]
        ret

        .Lswitch.table.new_result:
        .long   65535
        .long   0
        .long   1
        .long   2147483648
        ```
        HRESULT(match order {
            Some(cmp::Ordering::Less) => 0xFFFF,
            Some(cmp::Ordering::Equal) => 0,
            Some(cmp::Ordering::Greater) => 1,
            None => 1 << 31,
        })
        */
        HRESULT(match order {
            Some(order) => order as i16 as u16 as i32,
            None => 1 << 31,
        })
    }
}

impl From<LPARAM> for CompareIDs {
    fn from(val: LPARAM) -> Self {
        let v = val.0 as u32;
        CompareIDs {
            column: (v & SHCIDS_COLUMNMASK as u32) as u16,
            flags: (v >> 16) as u16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_result() {
        use cmp::Ordering::*;

        // Test Less (-1)
        let hres = CompareIDs::to_result(Some(Less));
        assert_eq!(hres.0, 0xFFFF);

        // Test Equal (0)
        let hres = CompareIDs::to_result(Some(Equal));
        assert_eq!(hres.0, 0);

        // Test Greater (1)
        let hres = CompareIDs::to_result(Some(Greater));
        assert_eq!(hres.0, 1);

        // Test None (fallback to -1)
        let hres = CompareIDs::to_result(None);
        assert_eq!(hres.0 as u32, 0x80000000);
    }
}
