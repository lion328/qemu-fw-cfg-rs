/* 
   Copyright 2021 Waritnan Sookbuntherng

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#![cfg_attr(not(feature = "alloc"), no_std)]
#![feature(asm)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{convert::TryInto, mem::size_of, str::from_utf8};

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), path = "x86.rs")]
mod arch;

mod selector_keys {
    pub const SIGNATURE: u16 = 0x0000;
    pub const ID: u16 = 0x0001;
    pub const DIR: u16 = 0x0019;
}

const SIGNATURE_DATA: &'static [u8] = b"QEMU";

#[derive(Debug)]
#[non_exhaustive]
pub enum FwCfgBuilderError {
    InvalidSignature,
}

pub struct FwCfgBuilder {
    prefer_dma: bool,
}

impl FwCfgBuilder {
    pub fn new() -> Self {
        Self {
            prefer_dma: true,
        }
    }

    pub fn with_prefer_dma(self, preference: bool) -> Self {
        Self {
            prefer_dma: preference,
        }
    }

    /// Build [`FwCfg`] from the builder. This is unsafe since there is no verification
    /// that this running inside QEMU before accessing I/O ports.
    pub unsafe fn build(self) -> Result<FwCfg, FwCfgBuilderError> {
        let mut signature = [0u8; SIGNATURE_DATA.len()];
        arch::write_selector(selector_keys::SIGNATURE);
        arch::read_data(&mut signature);

        if signature != SIGNATURE_DATA {
            return Err(FwCfgBuilderError::InvalidSignature);
        }

        let use_dma = if self.prefer_dma {
            let id = {
                let mut buf = [0u8; size_of::<u32>()];
                arch::write_selector(selector_keys::ID);
                arch::read_data(&mut buf);

                u32::from_le_bytes(buf)
            };

            (id >> 1) & 1 == 1
        } else {
            false
        };

        Ok(FwCfg {
            use_dma,
        })
    }
}

pub struct FwCfg {
    use_dma: bool,
}

impl FwCfg {
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

    pub fn find_file<'a>(&self, name: &'a str) -> Option<FwCfgFile<'a>> {
        let mut entries = [(name, None)];
        self.find_files(&mut entries);
        entries[0].1.take()
    }

    pub fn read_file_to_buffer<'a>(&self, file: &FwCfgFile<'a>, buffer: &mut [u8]) {
        self.select(file.key);
        self.read(&mut buffer[..file.size]);
    }

    #[cfg(feature = "alloc")]
    pub fn read_file<'a>(&self, file: &FwCfgFile<'a>) -> Vec<u8> {
        let mut buf = vec![0u8; file.size];
        self.select(file.key);
        self.read(&mut buf);
        buf
    }

    fn select(&self, key: u16) {
        unsafe {
            if self.use_dma {
                unimplemented!()
            } else {
                arch::write_selector(key);
            }
        }
    }

    fn read(&self, buffer: &mut [u8]) {
        unsafe {
            if self.use_dma {
                unimplemented!()
            } else {
                arch::read_data(buffer);
            }
        }
    }
}

const FW_CFG_FILE_SIZE: usize = 64;

pub struct FwCfgFile<'a> {
    size: usize,
    key: u16,
    name: &'a str, 
}

impl<'a> FwCfgFile<'a> {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    fn from_bytes(bytes: &'a [u8; FW_CFG_FILE_SIZE]) -> Self {
        Self {
            size: u32::from_be_bytes(bytes[..=3].try_into().unwrap()) as usize,
            key: u16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
            name: from_utf8(&bytes[9..]).unwrap(),
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
