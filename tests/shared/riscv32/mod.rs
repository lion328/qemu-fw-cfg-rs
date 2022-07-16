use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};
use qemu_fw_cfg::FwCfg;

static EXIT: AtomicPtr<u32> = AtomicPtr::new(null_mut());
static UART: AtomicPtr<u8> = AtomicPtr::new(null_mut());
static FW_CFG: AtomicPtr<()> = AtomicPtr::new(null_mut());

#[riscv_rt::entry]
fn main(_hart_id: usize, fdt_address: usize) -> ! {
    let fdt = unsafe { fdt::Fdt::from_ptr(fdt_address as _).unwrap() };
    find_compatible_reg(&fdt, "sifive,test1", &EXIT);
    find_compatible_reg(&fdt, "ns16550a", &UART);
    find_compatible_reg(&fdt, "qemu,fw-cfg-mmio", &FW_CFG);
    crate::main();
    exit(0)
}

fn find_compatible_reg<T>(fdt: &fdt::Fdt, with: &str, ptr: &AtomicPtr<T>) {
    ptr.store(
        fdt.find_compatible(&[with])
            .unwrap()
            .reg()
            .unwrap()
            .next()
            .unwrap()
            .starting_address as _,
        Ordering::Release,
    )
}

#[no_mangle]
pub extern "C" fn exit(status: u8) -> ! {
    unsafe {
        let ptr = EXIT.load(Ordering::Acquire);
        ptr.write_volatile((status as u32) << 16 | 0x3333);
    }
    loop {}
}

pub struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let uart = UART.load(Ordering::Acquire);
        for b in s.bytes() {
            unsafe {
                uart.write_volatile(b);
            }
        }
        Ok(())
    }
}

pub unsafe fn fw_cfg() -> FwCfg {
    FwCfg::new_memory_mapped(FW_CFG.load(Ordering::Acquire)).unwrap()
}
