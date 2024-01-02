#![no_std]
#![feature(abi_x86_interrupt)]

mod interrupts;

pub fn init() {
    interrupts::init_idt();
}
