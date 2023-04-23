#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use core::{mem::size_of, slice};

use zerocopy::{AsBytes, FromBytes};

#[derive(Debug, Clone, Copy, AsBytes, FromBytes)]
#[repr(C)]
pub struct Entry {
    pub code_offset: u32,
    pub sp_offset: u32,
    pub sp_reg: u8,
    pub fp_offset: u8,
    pub ra_offset: u8,
    pub flag: u8,
}

impl Entry {
    pub fn to_bytes(&self) -> &[u8] {
        <_ as AsBytes>::as_bytes(self)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Header {
    pub unwind: *const Entry,
    pub unwind_len: usize,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub text_start: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    pub pc: usize,
    pub sp: usize,
    pub fp: usize,
}

pub struct FirstFrame {
    pub ra: usize,
    pub frame: CallFrame,
}

pub unsafe fn unwind(
    header: &'static Header,
    context: &Context,
    first_frame: FirstFrame,
    debug: &mut dyn FnMut(CallFrame),
) {
    let len = header.unwind_len / size_of::<Entry>();
    let entries = unsafe { slice::from_raw_parts(header.unwind, len) };

    fn load(addr: usize) -> usize {
        unsafe { (addr as *const usize).read() }
    }

    (|| -> Option<()> {
        let mut frame = first_frame.frame;
        let mut is_top = true;

        loop {
            debug(frame);

            let offset: usize = frame.pc - context.text_start;
            let offset: u32 = offset.try_into().ok()?;
            let index = entries.partition_point(|e| e.code_offset <= offset);

            let entry = entries.get(index.checked_sub(1)?)?;

            if (entry.flag & 1) == 0 {
                break None;
            }

            let sp_base = match entry.sp_reg {
                2 => frame.sp,
                8 => frame.fp,
                _ => break None,
            };

            frame.sp = sp_base.wrapping_add(entry.sp_offset as usize);

            frame.fp = match entry.fp_offset {
                u8::MAX => frame.fp,
                off => load(frame.sp.wrapping_sub(off as usize)),
            };

            frame.pc = match entry.ra_offset {
                u8::MAX => {
                    if is_top {
                        first_frame.ra
                    } else {
                        break None;
                    }
                }
                off => load(frame.sp.wrapping_sub(off as usize)),
            };

            is_top = false;
        }
    })();
}
