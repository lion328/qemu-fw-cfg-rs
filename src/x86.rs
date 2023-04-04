use core::arch::asm;

// https://gitlab.com/qemu-project/qemu/-/blob/v7.0.0/docs/specs/fw_cfg.txt#L79
const IO_PORT_SELECTOR: u16 = 0x510;
const IO_PORT_DATA: u16 = 0x511;
const IO_PORT_DMA_ADDRESS: u16 = 0x514;

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

unsafe fn out_u32(address: u16, data: u32) {
    asm!(
        "out dx, eax",
        in("dx") address,
        in("eax") data,
    );
}

pub(crate) unsafe fn write_selector(key: u16) {
    out_u16(IO_PORT_SELECTOR, key);
}

pub(crate) unsafe fn read_data(buffer: &mut [u8]) {
    for i in buffer {
        *i = in_u8(IO_PORT_DATA);
    }
}

pub(crate) unsafe fn start_dma(access: &crate::FwCfgDmaAccess) {
    let address = access as *const crate::FwCfgDmaAccess as u64;
    // https://gitlab.com/qemu-project/qemu/-/blob/v7.0.0/docs/specs/fw_cfg.txt#L167
    // The DMA address register is 64-bit and big-endian,
    // but I/O ports only support 32-bit writes.
    let port_high = IO_PORT_DMA_ADDRESS;
    let port_low = IO_PORT_DMA_ADDRESS + 4;
    let address_high = (address >> 32) as u32;
    let address_low = address as u32;
    out_u32(port_high, address_high.to_be());
    // Write the lower bits last as this is what triggers DMA, do it last
    out_u32(port_low, address_low.to_be());
}
