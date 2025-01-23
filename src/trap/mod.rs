use core::{arch::global_asm, usize};
use riscv::{
    self,
    interrupt::{supervisor::Exception, Trap},
    register::{scause, sstatus::Sstatus, stval, stvec},
    ExceptionNumber,
};

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    unsafe {
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
}

#[repr(C)]
pub struct TrapContext {
    x: [usize; 32],
    sstatus: Sstatus,
    sepc: usize,
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    println!("into trap_handler");
    let scause = scause::read();
    let stval = stval::read();

    // use riscv::ExceptionNumber;
    // let t = riscv::interrupt::machine::Exception::UserEnvCall.number();
    // // let t = t.number();

    match scause.cause() {
        Trap::Exception(en) => match Exception::from_number(en) {
            Ok(Exception::UserEnvCall) => {
                cx.sepc += 4;
            }
            _ => panic!("Unsupported exception {:?}", en),
        },
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }

    cx
}
