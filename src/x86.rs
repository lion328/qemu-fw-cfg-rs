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
