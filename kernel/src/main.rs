#![no_std]
#![no_main]

use core::arch::asm;

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config
};
bootloader_api::entry_point!(kernel_main, config = &CONFIG);

const PORT: u16 = 0x3F8;

unsafe fn write_hello() {
    write_serial(104);
    write_serial(101);
    write_serial(108);
    write_serial(108);
    write_serial(111);
    write_serial(13);
    write_serial(10);
}

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    unsafe {
        init_serial();
        write_hello();
        write_hello();
        write_hello();
    };
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

unsafe fn is_transmit_empty() -> u8 {
    inb(PORT + 5) & 0x20
}

unsafe fn write_serial(a: u8) {
    while is_transmit_empty() == 0 {}

    outb(PORT, a);
}

unsafe fn init_serial() {
    outb(PORT + 1, 0x00); // Disable all interrupts
    outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
    outb(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    outb(PORT + 1, 0x00); //                  (hi byte)
    outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
    outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
    outb(PORT + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

    // Check if serial is faulty (i.e: not same byte as sent)
    if inb(PORT + 0) != 0xAE {
        panic!("Serial is faulty");
    }

    // If serial is not faulty set it in normal operation mode
    // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
    outb(PORT + 4, 0x0F);
}

#[inline]
unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val);
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    #[allow(unused_assignments)]
    let mut ret: u8 = 0;
    asm!( "in ax, dx", in("dx") port, out("al") ret);
    ret
}
