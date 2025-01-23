#![no_std]
#![no_main]

use core::arch::global_asm;
use kernel::Kernel;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    // App::run()

    Kernel::new().run()
}
