#![no_std]
#![no_main]

use conquer_once::spin::OnceCell;
use printk::LockedPrintk;

const CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};
bootloader_api::entry_point!(kernel_main, config = &CONFIG);

pub static PRINTK: OnceCell<LockedPrintk> = OnceCell::uninit();

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let fb = boot_info.framebuffer.as_mut().unwrap();
    let fb_info = fb.info();

    let kernel_logger =
        PRINTK.get_or_init(move || printk::LockedPrintk::new(fb.buffer_mut(), fb_info));
    log::set_logger(kernel_logger).expect("logger already set");
    log::set_max_level(log::LevelFilter::Trace);

    log::info!("Welcome to RosaOS");

    kernel::init();

    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::info!("KERNEL PANIC {}", info);

    loop {}
}
