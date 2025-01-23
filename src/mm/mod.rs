// mod heap_allocator;

use sv39::page_table::PageTableError;

// pub use heap_allocator::*;
pub mod frame_allocator;
pub mod heap_allocator;
pub mod sv39;

pub const PAGE_OFFSET: usize = 12;

pub const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_LENGTH: usize = 512;
const MAX_FRAME: usize = (1 << 44) - 1;

#[macro_export]
macro_rules! impl_from_to_usize {
    ($name: ident) => {
        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                Self(value)
            }
        }

        impl From<$name> for usize {
            fn from(value: $name) -> Self {
                value.0
            }
        }
        impl $name {
            pub fn value(&self) -> usize {
                self.0
            }
        }
    };
}

/// representing a virtual address
///
/// | vpns (27) | offset (12) |
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VirtualAddress(usize);
impl_from_to_usize!(VirtualAddress);

/// representing a virtual page number vpn,
/// 27 bits available
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VirtualPageNumber(usize);
impl_from_to_usize!(VirtualPageNumber);

impl VirtualPageNumber {
    fn vpns(&self) -> Result<VPNS, PageTableError> {
        let vpns = self.0;

        let flags = (1 << 9) - 1;
        let vpns = VPNS([vpns & flags, (vpns >> 9) & flags, (vpns >> 18) & flags]);

        if vpns.0.iter().any(|v| *v >= 512) {
            Err(PageTableError::InvalidVPN(
                (vpns.get2() << 18) & (vpns.get1() << 9) & vpns.get0(),
            ))
        } else {
            Ok(vpns)
        }
    }
}

/// representing a physical frame, physical address with 4KB size
/// note: it has no offset bits, 44 bits availiable
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame(usize);
impl_from_to_usize!(Frame);

impl From<PhysicalAddress> for Frame {
    fn from(value: PhysicalAddress) -> Self {
        Self(value.value())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// representing a physical address
///
/// | physical page number(44) | offset (12) |
pub struct PhysicalAddress(usize);

impl_from_to_usize!(PhysicalAddress);

impl From<Frame> for PhysicalAddress {
    fn from(value: Frame) -> Self {
        Self(value.value())
    }
}

impl PhysicalAddress {
    fn into_frame(self) -> Frame {
        (self.value() >> PAGE_OFFSET).into()
    }
}

impl VirtualAddress {
    fn vpns(self) -> Result<VPNS, PageTableError> {
        self.into_virtual_page_number().vpns()
    }

    fn into_virtual_page_number(self) -> VirtualPageNumber {
        (self.value() >> PAGE_OFFSET).into()
    }
}

struct VPNS([usize; 3]);

impl VPNS {
    pub fn get2(&self) -> usize {
        self.0[2]
    }
    pub fn get1(&self) -> usize {
        self.0[1]
    }
    pub fn get0(&self) -> usize {
        self.0[0]
    }
}
