use rvind_unwinder::Entry;

use crate::analysis::{OrigSpReg, UnwindStep};

fn convert(offset: i64, unwind: UnwindStep) -> Option<Entry> {
    let offset: u32 = offset.try_into().ok()?;

    let convert = |x: Option<i64>| {
        if let Some(x) = x {
            (-x).try_into().ok()
        } else {
            Some(u8::MAX)
        }
    };

    Some(Entry {
        code_offset: offset,
        sp_offset: unwind.sp_offset.try_into().ok()?,
        sp_reg: match unwind.sp_reg {
            OrigSpReg::Sp => 2,
            OrigSpReg::Fp => 8,
        },
        fp_offset: convert(unwind.fp_offset)?,
        ra_offset: convert(unwind.ra_offset)?,
        flag: 1,
    })
}

pub fn convert_unwind(offset: i64, unwind: Option<UnwindStep>) -> Entry {
    unwind.and_then(|u| convert(offset, u)).unwrap_or(Entry {
        code_offset: offset
            .try_into()
            .expect("Code offset should not overflow 4 GiB"),
        sp_offset: 0,
        sp_reg: 0,
        fp_offset: 0,
        ra_offset: 0,
        flag: 0,
    })
}
