#![no_std]
#![no_main]

#![feature(global_asm)]
#![feature(asm)]
#![feature(default_alloc_error_handler)]

use qemu_fw_cfg::FwCfgBuilder;

mod shared;

const INPUT: &'static [u8] = include_bytes!("input.txt");

#[no_mangle]
fn main() {
    let fw_cfg = unsafe {
        FwCfgBuilder::new().with_prefer_dma(false).build().unwrap()
    };

    let file = fw_cfg.find_file("opt/input.txt").unwrap();
    assert_eq!(INPUT, fw_cfg.read_file(&file));

    let mut buffer = [0u8; INPUT.len()];
    fw_cfg.read_file_to_buffer(&file, &mut buffer);
    assert_eq!(INPUT, buffer);

    assert!(fw_cfg.find_file("opt/not_found.txt").is_none())
}
