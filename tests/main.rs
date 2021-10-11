#![no_std]
#![no_main]

#![feature(global_asm)]
#![feature(asm)]
#![feature(default_alloc_error_handler)]

use qemu_fw_cfg::FwCfg;

mod shared;

const INPUT: &'static [u8] = include_bytes!("input.txt");

#[no_mangle]
fn main() {
    let fw_cfg = unsafe {
        FwCfg::new().unwrap()
    };

    let file = fw_cfg.find_file("opt/input.txt").unwrap();
    assert_eq!(INPUT, fw_cfg.read_file(&file));

    let mut buffer = [0u8; INPUT.len()];
    fw_cfg.read_file_to_buffer(&file, &mut buffer);
    assert_eq!(INPUT, buffer);

    assert!(fw_cfg.find_file("opt/not_found.txt").is_none());

    assert!(fw_cfg.find_file("opt/567890123456789012345678901234567890123456789012345").is_some());
}
