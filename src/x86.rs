use crate::FwCfg;
use core::arch::asm;

const IO_PORT_SELECTOR: u16 = 0x510;
const IO_PORT_DATA: u16 = 0x511;

unsafe fn in_u8(address: u16) -> u8 {
    let ret: u8;
    asm!(
        "in al, dx",
        out("al") ret,
        in("dx") address,
    );
    ret
}

unsafe fn out_u16(address: u16, data: u16) {
    asm!(
        "out dx, ax",
        in("dx") address,
        in("ax") data,
    );
}

impl FwCfg {
    pub(crate) unsafe fn write_selector(key: u16) {
        out_u16(IO_PORT_SELECTOR, key);
    }

    pub(crate) unsafe fn read_data(buffer: &mut [u8]) {
        for i in buffer {
            *i = in_u8(IO_PORT_DATA);
        }
    }
}
