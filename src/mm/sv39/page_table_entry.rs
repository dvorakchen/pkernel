use bitflags::bitflags;

use crate::mm::Frame;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Flags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct PTE(usize);

impl PTE {
    pub(crate) fn new(frame: Frame, flags: Flags) -> Self {
        let pte = (frame.value() & ((1 << 44) - 1)) << 10 | flags.bits() as usize;
        PTE(pte)
    }

    pub fn is_valid(&self) -> bool {
        self.0 as u8 & Flags::V.bits() == Flags::V.bits()
    }

    /// get the physical page number,
    /// if this is not a leaf page table, the return is [PhysicalAddress](crate::mm::PhysicalAddress), or a [Frame]
    pub fn ppn(&self) -> Frame {
        ((self.0 >> 10) & ((1 << 44) - 1)).into()
    }
}
