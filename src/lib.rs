//! A Rust library for reading fw_cfg from QEMU.
//!
//! # Supported architectures
//!
//! As of today, this crate only supported x86 and x86_64. However, it is possible
//! to add support for other platforms, such as ARM.
//!
//! # Examples
//! ```
//! use qemu_fw_cfg::FwCfg;
//!
//! // Verify that we are inside QEMU.
//! if running_in_qemu() {
//!     // Create a new `FwCfg` instance.
//!     let fw_cfg = unsafe { FwCfg::new().unwrap() };
//!     // Retrieve information of a file.
//!     let file = fw_cfg.find_file("etc/igd-opregion").unwrap();
//!     // Read data from the file.
//!     let data = fw_cfg.read_file(&file);
//! }
//! ```

#![no_std]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::{convert::TryInto, mem::size_of, str::from_utf8};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "x86.rs"]
mod arch;

mod selector_keys {
    pub const SIGNATURE: u16 = 0x0000;
    pub const DIR: u16 = 0x0019;
}

const SIGNATURE_DATA: &[u8] = b"QEMU";

/// An enum type for [`FwCfg`] errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum FwCfgError {
    /// Invalid signature returned from QEMU fw_cfg I/O port
    InvalidSignature,
}

/// A struct for accessing QEMU fw_cfg.
#[derive(Debug)]
pub struct FwCfg(());

impl FwCfg {
    /// Build `FwCfg` from the builder.
    ///
    /// # Safety
    /// This is unsafe since there is no verification that this running inside QEMU
    /// before accessing I/O ports. Caller must verify this condition first.
    pub unsafe fn new() -> Result<FwCfg, FwCfgError> {
        let mut signature = [0u8; SIGNATURE_DATA.len()];
        Self::write_selector(selector_keys::SIGNATURE);
        Self::read_data(&mut signature);

        if signature != SIGNATURE_DATA {
            return Err(FwCfgError::InvalidSignature);
        }

        Ok(FwCfg(()))
    }

    /// Find one or more files by their name.
    ///
    /// Each tuple in `entries` must consisted of file name and a space for
    /// `Option<FwCfgFile>`. If a file is found, the result will be stored by
    /// replacing the value in `Option<FwCfgFile>` of the corresponding tuple,
    /// otherwise it will retained the same value as before.
    ///
    /// # Examples
    /// ```
    /// use qemu_fw_cfg::FwCfg;
    ///
    /// let fw_cfg = unsafe { FwCfg::new().unwrap() };
    /// let mut files = [
    ///     ("etc/igd-opregion", None),
    ///     ("opt/another/file.txt", None),
    /// ];
    /// fw_cfg.find_files(&mut files);
    /// ```
    pub fn find_files<'a, 'b>(&self, entries: &'a mut [(&'b str, Option<FwCfgFile<'b>>)]) {
        self.select(selector_keys::DIR);

        let count = {
            let mut buf = [0u8; size_of::<u32>()];
            self.read(&mut buf);
            u32::from_be_bytes(buf)
        };

        let mut buf = [0u8; FW_CFG_FILE_SIZE];

        for _ in 0..count {
            self.read(&mut buf);
            let file = FwCfgFile::from_bytes(&buf);
            let mut changed = false;

            for (name, ret) in entries.iter_mut() {
                if file.name() == *name {
                    ret.replace(file.with_name(*name));
                    changed = true;
                }
            }

            if changed && entries.iter().all(|entry| entry.1.is_some()) {
                return;
            }
        }
    }

    /// Find a single file by its name. Returns `None` if the file is missing.
    ///
    /// # Examples
    /// ```
    /// use qemu_fw_cfg::FwCfg;
    ///
    /// let fw_cfg = unsafe { FwCfg::new().unwrap() };
    /// let file = fw_cfg.find_file("etc/igd-opregion").unwrap();
    /// ```
    pub fn find_file<'a>(&self, name: &'a str) -> Option<FwCfgFile<'a>> {
        let mut entries = [(name, None)];
        self.find_files(&mut entries);
        entries[0].1.take()
    }

    /// Read a file and fill its data in `buffer`.
    ///
    /// If the size of `buffer` is greater or equals to the size of the file,
    /// then it will fill the entire data in `buffer[0..file.size()]`, otherwise
    /// it will only fill up to `buffer.len()`.
    pub fn read_file_to_buffer<'a>(&self, file: &FwCfgFile<'a>, buffer: &mut [u8]) {
        let len = file.size.min(buffer.len());
        self.select(file.key);
        self.read(&mut buffer[..len]);
    }

    /// Read a file and return the data in `Vec<u8>`.
    #[cfg(feature = "alloc")]
    pub fn read_file<'a>(&self, file: &FwCfgFile<'a>) -> Vec<u8> {
        let mut buf = vec![0u8; file.size];
        self.select(file.key);
        self.read(&mut buf);
        buf
    }

    fn select(&self, key: u16) {
        unsafe {
            Self::write_selector(key);
        }
    }

    fn read(&self, buffer: &mut [u8]) {
        unsafe {
            Self::read_data(buffer);
        }
    }
}

const FW_CFG_FILE_SIZE: usize = 64;

/// A struct that contains information of a fw_cfg file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FwCfgFile<'a> {
    size: usize,
    key: u16,
    name: &'a str,
}

impl<'a> FwCfgFile<'a> {
    /// The size of this file.
    pub fn size(&self) -> usize {
        self.size
    }

    /// The name of this file.
    pub fn name(&self) -> &'a str {
        self.name
    }

    fn from_bytes(bytes: &'a [u8; FW_CFG_FILE_SIZE]) -> Self {
        let name_bytes = &bytes[8..];
        let name_len = name_bytes
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(name_bytes.len());

        Self {
            size: u32::from_be_bytes(bytes[..=3].try_into().unwrap()) as usize,
            key: u16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
            name: from_utf8(&name_bytes[..name_len]).unwrap(),
        }
    }

    fn with_name<'b: 'a>(&self, name: &'b str) -> FwCfgFile<'b> {
        FwCfgFile {
            size: self.size,
            key: self.key,
            name,
        }
    }
}
