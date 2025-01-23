#![no_std]
#![feature(alloc_error_handler)]

mod lang_item;
mod sbi;
#[macro_use]
mod console;
pub mod mm;
mod trap;

use core::arch::asm;

use mm::{
    frame_allocator::{FrameAllocator, SharedFrameAllocator},
    heap_allocator::init_heap,
    sv39::{
        page_table::{MapBlock, RootPageTable},
        page_table_entry::Flags,
    },
    PAGE_OFFSET, PAGE_SIZE,
};
use riscv::{
    asm::wfi,
    register::{satp::Mode, time},
};

extern crate alloc;

extern "C" {
    fn ekernel();
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
}

pub struct Kernel {
    page_table: RootPageTable,
    frame_alloc: SharedFrameAllocator,
}

impl Kernel {
    pub fn new() -> Self {
        // clear_bss();
        init_heap();

        let allocator = Self::init_frame_allocator();
        let page_table = Self::init_kernel_page_table(allocator.clone());

        trap::init();
        println!("\x1b[31mhello world\x1b[0m");

        Self {
            frame_alloc: allocator,
            page_table,
        }
    }

    fn init_frame_allocator() -> SharedFrameAllocator {
        const MEMORY_END: usize = 0x8800_0000;
        FrameAllocator::new(ekernel as usize, MEMORY_END).unwrap()
    }

    fn init_kernel_page_table(allocator: SharedFrameAllocator) -> RootPageTable {
        let mut rpt = RootPageTable::new(allocator).unwrap();

        macro_rules! map_kernel_memory {
            ($start: expr, $end: expr, $flags: expr) => {
                println!("start: {:X}, end: {:X}", $start, $end);
                let count = ($end - $start) / PAGE_SIZE;
                for i in 0..count {
                    rpt.map(MapBlock::new(
                        ((($start as usize) + i * PAGE_SIZE) >> PAGE_OFFSET),
                        mm::sv39::page_table::MapType::Identical,
                        Flags::V | $flags,
                    ))
                    .unwrap();
                }
            };
        }

        map_kernel_memory!(stext as usize, etext as usize, Flags::X | Flags::R);
        map_kernel_memory!(srodata as usize, erodata as usize, Flags::R);
        map_kernel_memory!(sdata as usize, edata as usize, Flags::W | Flags::R);
        map_kernel_memory!(sbss as usize, ebss as usize, Flags::W | Flags::R);

        rpt
    }

    pub fn run(self) -> ! {
        println!("run");
        self.use_sv39();
        println!("sv39 page mode");
        println!("WFI");
        wfi();
        panic!("shutdown machine");
    }

    pub fn use_sv39(&self) {
        // let satp: usize = self.page_table.satp_token().into();
        // riscv::register::satp::write(satp);
        // let ppn = self.page_table.address.into();
        unsafe {
            let tok: usize = self.page_table.satp_token().into();
            asm!(concat!("csrw satp, {0}"), in(reg) tok);
            // let res = riscv::register::satp::try_set(Mode::Sv39, 0, ppn);
            println!("set satp");
            asm!("SFENCE.VMA");
        }
    }
}

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize..ebss as usize).for_each(|a| unsafe {
        let v = a as *const u8;
        if *v != 0 {
            println!("{} no zero", 0);
        }
        (a as *mut u8).write_volatile(0)
    });
}
