#[cfg(target_arch = "x86")]
#[path = "i686/mod.rs"]
mod arch;

pub use arch::*;

use core::{alloc::{GlobalAlloc, Layout}, cell::UnsafeCell, ops::Sub, panic::PanicInfo, ptr::null_mut, sync::atomic::{AtomicUsize, Ordering}};

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print!("{}\n", format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        $crate::shared::Writer.write_fmt(format_args!($($arg)*)).unwrap()
    });
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    exit(1)
}

const HEAP_SIZE: usize = 16 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LinearAllocator = LinearAllocator {
    heap: UnsafeCell::new([0u8; HEAP_SIZE]),
    next: AtomicUsize::new(0),
};

#[repr(C, align(4096))]
struct LinearAllocator {
    heap: UnsafeCell<[u8; HEAP_SIZE]>,
    next: AtomicUsize,
}

unsafe impl Sync for LinearAllocator {}

unsafe impl GlobalAlloc for LinearAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mask_offset = align - 1;
        let mask_valid_addr = !mask_offset;
        let mut next = 0;

        if self.next.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old_next| {
            next = if old_next & mask_offset == 0 {
                old_next
            } else {
                (old_next & mask_valid_addr) + align
            };

            let end = next + size;
            if end >= HEAP_SIZE {
                return None;
            }

            Some(end)
        }).is_err() {
            return null_mut();
        }

        (self.heap.get() as *mut u8).add(next)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}
