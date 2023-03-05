// #![feature(default_alloc_error_handler)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod threshold_examiner;
mod rules;

// use threshold_examiner::{Action, Examiner};
use core::{panic::PanicInfo, ffi::c_void};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

use alloc::alloc::*;

extern "C" {
    fn malloc(layout_size: u32) -> *mut c_void;
    fn free(ptr: *mut c_void);

}

/// The global allocator type.
#[derive(Default)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
         malloc(layout.size() as u32) as *mut u8
     }
     unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
         free(ptr as *mut c_void);
     }
}

/// If there is an out of memory error, just panic.
// #[alloc_error_handler]
// fn my_allocator_error(_layout: Layout) -> ! {
//     panic!("out of memory");
// }

/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;


#[no_mangle]
pub unsafe extern "C" fn test_func_for_bindgen() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::prelude::*;

    #[test]
    fn test_main_handler_handles() {}

    #[test]
    fn test_stats_provider_provides() {}
}
