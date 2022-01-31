use core::arch::{asm, global_asm};

global_asm!(include_str!("boot.asm"));

unsafe fn outb(port: u16, byte: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") byte,
    );
}

#[no_mangle]
pub extern "C" fn exit(status: u8) -> ! {
    unsafe {
        outb(0x501, status);
    }

    loop {}
}

pub struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            unsafe {
                outb(0x3F8, c);
            }
        }
        Ok(())
    }
}
