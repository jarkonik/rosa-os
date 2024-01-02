#![no_std]
#![no_main]

use core::{arch::asm, ffi::c_void, ptr::null_mut};

use bootloader_api::info::MemoryRegionKind;
use conquer_once::spin::OnceCell;
use printk::LockedPrintk;

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};
bootloader_api::entry_point!(kernel_main, config = &CONFIG);

const PORT: u16 = 0x3F8;

pub static PRINTK: OnceCell<LockedPrintk> = OnceCell::uninit();

unsafe fn write_hello() {
    write_serial(104);
    write_serial(101);
    write_serial(108);
    write_serial(108);
    write_serial(111);
    write_serial(13);
    write_serial(10);
}

static mut MEMORY_START: *mut u64 = null_mut();
static mut MEMORY_PTR: *mut u64 = null_mut();
static mut MEMORY_END: *mut u64 = null_mut();

#[allow(dead_code)]
unsafe fn kmalloc(size: usize) -> *mut c_void {
    if MEMORY_PTR.add(size) >= MEMORY_END {
        return null_mut();
    }

    let result = unsafe { MEMORY_PTR };
    unsafe { MEMORY_PTR = MEMORY_PTR.add(size) }
    result as *mut c_void
}

fn kfree(ptr: *mut c_void) {}

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let fb = boot_info.framebuffer.as_mut().unwrap();
    let fb_info = fb.info();

    let kernel_logger =
        PRINTK.get_or_init(move || printk::LockedPrintk::new(fb.buffer_mut(), fb_info));
    log::set_logger(kernel_logger).expect("logger already set");
    log::set_max_level(log::LevelFilter::Trace);

    log::info!("Welcome to RosaOS");

    kernel::init();

    for region in boot_info
        .memory_regions
        .iter()
        .filter(|r| matches!(r.kind, MemoryRegionKind::Usable))
    {
        // There may be more than one filter, currently we assume there is only one here
        unsafe {
            MEMORY_START = (boot_info.physical_memory_offset.into_option().unwrap() + region.start)
                as *mut u64;
            MEMORY_PTR = (boot_info.physical_memory_offset.into_option().unwrap() + region.start)
                as *mut u64;
            MEMORY_END =
                (boot_info.physical_memory_offset.into_option().unwrap() + region.end) as *mut u64;
        }
        break;
    }

    unsafe {
        let ptr = kmalloc(1000 * 8);
        if ptr == null_mut() {
            panic!();
        }

        log::info!(
            "{:?} {:?} {:?} {:?}",
            MEMORY_START,
            MEMORY_PTR,
            MEMORY_END,
            ptr
        );

        for i in 1..1000 {
            *(ptr.add(i * 8) as *mut u64) = i as u64;
        }

        for i in 1..1000 {
            let x = *(ptr.add(i * 8) as *mut u64);
            log::info!("{}", x);
        }

        init_serial();
        write_hello();
        write_hello();
        write_hello();
    };
    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::info!("KERNEL PANIC {}", info);

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
    outb(PORT, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    outb(PORT + 1, 0x00); //                  (hi byte)
    outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
    outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
    outb(PORT, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

    // Check if serial is faulty (i.e: not same byte as sent)
    if inb(PORT) != 0xAE {
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
