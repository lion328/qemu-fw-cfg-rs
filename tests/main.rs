#![no_std]
#![no_main]
#![cfg_attr(feature = "alloc", feature(default_alloc_error_handler))]

use core::fmt::Write;

mod shared;

const DATA_INPUT_TXT: &'static [u8] = include_bytes!("input.txt");

#[cfg_attr(not(target_arch = "riscv32"), no_mangle)]
fn main() {
    let mut fw_cfg = unsafe { shared::fw_cfg() };

    // File exist
    let file_input_txt = fw_cfg.find_file("opt/input.txt").unwrap();

    // File not exist
    assert!(fw_cfg.find_file("opt/not_found.txt").is_none());

    // Long file name
    fw_cfg
        .find_file("opt/567890123456789012345678901234567890123456789012345")
        .unwrap();

    // Multiple files
    let mut files = [
        ("opt/input.txt", None),
        ("opt/not_found.txt", None),
        ("opt/input.txt", None),
        ("opt/not_found.txt", Some(file_input_txt.clone())),
    ];
    fw_cfg.find_files(&mut files);
    assert_eq!(
        files.map(|i| i.1),
        [
            Some(file_input_txt.clone()),
            None,
            Some(file_input_txt.clone()),
            Some(file_input_txt.clone()),
        ]
    );

    // Read file
    #[cfg(feature = "alloc")]
    assert_eq!(DATA_INPUT_TXT, fw_cfg.read_file(&file_input_txt));

    // Read file with buffer
    let mut buffer = [0u8; DATA_INPUT_TXT.len()];
    fw_cfg.read_file_to_buffer(&file_input_txt, &mut buffer);
    assert_eq!(DATA_INPUT_TXT, buffer);

    // Small buffer
    let mut buffer = [0u8; DATA_INPUT_TXT.len() / 2];
    fw_cfg.read_file_to_buffer(&file_input_txt, &mut buffer);
    assert_eq!(DATA_INPUT_TXT[..buffer.len()], buffer);

    writeln!(shared::Writer, "✅ Test sucessful").unwrap();
}
