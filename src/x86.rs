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

const IO_PORT_SELECTOR: u16 = 0x510;
const IO_PORT_DATA: u16 = 0x511;
const IO_PORT_DMA: u16 = 0x514;

const EFLAGS_ID_MASK: u32 = 0x00200000;
const CPUID_QEMU_LEAF: u32 = 0x40000000;

pub fn is_inside_qemu() -> bool {
    if cpuid_supported() {
        let s = unsafe {
            cpuid(CPUID_QEMU_LEAF)
        };
        if &s == b"TCGTCGTCGTCG" || &s == b"KVMKVMKVM\0\0\0" {
            return true;
        }
    }
    
    // TODO: check ACPI

    false
}

#[cfg(target_arch = "x86_64")]
fn cpuid_supported() -> bool {
    let mut diff: u64;

    unsafe {
        asm!(
            "pushfq",
            "pushfq",
            "xor qword ptr [rsp], {0}",
            "popfq",
            "pushfq",
            "pop {1}",
            "xor {1}, [rsp]",
            "popfq",
            const EFLAGS_ID_MASK,
            out(reg) diff,
            options(preserves_flags),
        );
    }

    (diff as u32) & EFLAGS_ID_MASK != 0
}

#[cfg(target_arch = "x86_64")]
unsafe fn cpuid(leaf: u32) -> [u8; 12] {
    let mut buf = [0u8; 12];
    asm!(
        "push rbx",
        "cpuid",
        "mov dword ptr [{0}], ebx",
        "mov dword ptr [{0} + 1], edx",
        "mov dword ptr [{0} + 2], ecx",
        "pop rbx",
        in(reg) &mut buf,
        inout("eax") leaf => _,
        out("ecx") _,
        out("edx") _,
    );
    buf
}

pub unsafe fn write_selector(key: u16) {
    out_u16(IO_PORT_SELECTOR, key);
}

pub unsafe fn read_data(buffer: &mut [u8]) {
    for i in buffer {
        *i = in_u8(IO_PORT_DATA);
    }
}

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
