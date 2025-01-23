use alloc::string::ToString;
use hashbrown::HashSet;
use thiserror::Error;

use crate::impl_from_to_usize;

use crate::mm::{frame_allocator::SharedFrameAllocator, PAGE_TABLE_LENGTH};
use crate::mm::{Frame, PhysicalAddress, VirtualPageNumber};

use super::page_table_entry::{Flags, PTE};

pub struct SatpToken(usize);

impl_from_to_usize!(SatpToken);

#[derive(Debug, Error)]
pub enum PageTableError {
    #[error("no more physical")]
    NoMorePhysical,
    #[error("invalid virtual page number: {0}")]
    InvalidVPN(usize),
}

/// a struct that how map a virtual page number to frame
///
/// # Argument:
/// - vpn: virtual page number
/// - map_type: how to map
#[derive(Debug, Clone, Copy)]
pub struct MapBlock {
    vpn: VirtualPageNumber,
    map_type: MapType,
    permission: Flags,
}

impl MapBlock {
    pub fn new(vpn: usize, map_type: MapType, permission: Flags) -> Self {
        Self {
            vpn: vpn.into(),
            map_type,
            permission,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MapType {
    /// map directly, virtual page number to the same
    Identical,
    /// map to a frame
    UseFrame(Frame),
}

pub struct RootPageTable {
    pub address: PhysicalAddress,
    frames: HashSet<Frame>,
    allocator: SharedFrameAllocator,
}

impl RootPageTable {
    pub fn new(allocator: SharedFrameAllocator) -> Result<Self, PageTableError> {
        let frame = allocator
            .borrow_mut()
            .alloc()
            .ok_or(PageTableError::NoMorePhysical)?;

        let mut list = HashSet::new();
        list.insert(frame);

        Ok(Self {
            address: frame.into(),
            frames: list,
            allocator,
        })
    }

    /// map a [VirtualPageNumber] to a [Frame],
    /// if it already has a mapping, overwrite it
    pub fn map(&mut self, block: MapBlock) -> Result<(), PageTableError> {
        let vpns = block.vpn.vpns()?;

        // level 1 page tabl
        let mut lpt = LevelPageTable::new(self.address, self.allocator.clone());
        let mut frame = lpt.get_or_create(vpns.get2());
        self.frames.insert(frame);

        // level 2 page table
        lpt = LevelPageTable::new(frame.into(), self.allocator.clone());
        frame = lpt.get_or_create(vpns.get1());
        self.frames.insert(frame);

        lpt = LevelPageTable::new(frame.into(), self.allocator.clone());

        frame = match block.map_type {
            MapType::Identical => block.vpn.value().into(),
            MapType::UseFrame(f) => f,
        };
        let old = lpt.replace(vpns.get0(), PTE::new(frame, block.permission));
        self.frames.insert(frame);

        if old.is_valid() {
            self.allocator.borrow_mut().dealloc(old.ppn());
        }

        Ok(())
    }

    pub fn satp_token(&self) -> SatpToken {
        let mode = 8usize << 60;
        let asid = 0usize;
        let ppn = self.address.value() & ((1 << 44) - 1);

        (mode | asid | ppn).into()
    }
}

struct LevelPageTable {
    address: PhysicalAddress,
    alloc: SharedFrameAllocator,
}

impl LevelPageTable {
    fn new(physical_address: PhysicalAddress, alloc: SharedFrameAllocator) -> Self {
        Self {
            address: physical_address,
            alloc,
        }
    }

    /// get a frame by vpn
    ///
    /// # Argument:
    /// - vpn: one of three virtual page number (27 bits). it should be has 9 valid bits
    ///
    /// # Return:
    /// [Option], a frame that representing a physical frame without  offset
    fn get(&self, vpn: usize) -> Option<Frame> {
        if vpn >= PAGE_TABLE_LENGTH {
            panic!("wrong vpn: {}", PageTableError::InvalidVPN(vpn).to_string());
        }
        let ptes = self.ptes();
        let pte = &ptes[vpn];
        if !pte.is_valid() {
            return None;
        }

        Some(pte.ppn())
    }

    /// get a frame by vpn from this page table, if none, than create a frame
    /// if it has the frame, the argument frame would not be used, and returns the frame that existed
    ///
    /// # Argument:
    /// - vpn: one of three virtual page number
    ///
    /// # Return:
    /// returns [Some] indicates has frame, or None indicates has no frame
    fn get_or_create(&mut self, vpn: usize) -> Frame {
        let ptes = self.ptes_mut();
        let pte = &mut ptes[vpn];
        let frame = if pte.is_valid() {
            pte.ppn()
        } else {
            let frame = self
                .alloc
                .borrow_mut()
                .alloc()
                .expect(&PageTableError::NoMorePhysical.to_string());

            *pte = PTE::new(frame, Flags::V);
            frame
        };

        frame
    }

    fn replace(&mut self, vpn: usize, new_pte: PTE) -> PTE {
        let ptes = self.ptes_mut();
        let old = ptes[vpn];

        ptes[vpn] = new_pte;

        old
    }

    fn ptes(&self) -> &[PTE] {
        self.ptes_mut()
    }

    fn ptes_mut(&self) -> &mut [PTE] {
        unsafe {
            core::slice::from_raw_parts_mut(self.address.value() as *mut PTE, PAGE_TABLE_LENGTH)
        }
    }
}

impl Drop for RootPageTable {
    fn drop(&mut self) {
        let mut alloc = self.allocator.borrow_mut();
        self.frames.iter().for_each(|f| {
            alloc.dealloc(*f);
        });
    }
}
