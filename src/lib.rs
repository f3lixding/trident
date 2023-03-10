#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod rules;
mod threshold_examiner;

use alloc::boxed::Box;

#[allow(unused_imports)]
use core::{ffi::c_void, panic::PanicInfo};
use threshold_examiner::Examiner;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

use alloc::alloc::*;

extern "C" {
    fn malloc(layout_size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

/// The global allocator type.
#[derive(Default)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size() as usize) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut c_void);
    }
}

/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

/// C side constructs
#[repr(C)]
pub struct WrappedExaminer(*mut c_void);

impl Drop for WrappedExaminer {
    fn drop(&mut self) {
        let inner = self.0;
        unsafe {
            let inner = &mut *(inner as *mut Examiner);
            let _ = Box::from_raw(inner);
        }
    }
}

static MOISTURE_THRESHOLD: i32 = 30;

#[no_mangle]
/// Initializes an examiner and assigns its pointer to one that has been passed in from C side
///
/// # Arguments
///
/// * `_examiner_ptr` - A pointer to the wrapped struct that is really a tuple struct
/// The consumer of this api does not have to work with the inners of this struct outside of the
/// provided api
pub unsafe extern "C" fn initialize_examiner(mut _examiner_ptr: &mut *mut WrappedExaminer) -> i32 {
    let examiner = Examiner::new(MOISTURE_THRESHOLD);
    // sanity check
    if *(examiner.get_threshold()) != MOISTURE_THRESHOLD || *(examiner.get_latest_humd()) != 12 {
        return 0;
    }
    let examiner_raw_ptr = Box::leak(Box::new(examiner)) as *mut Examiner;
    *_examiner_ptr = Box::into_raw(Box::from(WrappedExaminer(examiner_raw_ptr as *mut c_void)));

    1
}

#[no_mangle]
/// Takes an initialized wrapped examiner and a humidity reading and evaluates all rules.
/// It will also call action function (i.e. turn on the pump) if the evaluations deems it
/// necessary.
/// Note that the action function is supplied statically during compile time via the use of extern
/// function.
///
/// # Arguments
///
/// * `examiner_ptr` - A pointer to an initialized wrapped examiner.
/// * `humd_reading` - An integer representing the humdity reading in percentage
pub unsafe extern "C" fn handle_humd_input(examiner_ptr: *mut WrappedExaminer, humd_reading: i32) {
    let examiner: &mut Examiner = {
        let inner = (*examiner_ptr).0;
        let inner = &mut *(inner as *mut Examiner);
        inner
    };

    // not sure what there is to do with the result at this point
    let _ = examiner.handle_humd_input(humd_reading);
}

#[no_mangle]
/// Takes an initialized wrapped examiner and frees it.
/// This is technically not needed here because we are not using std and the allocator we are using
/// are the same one used on C side. Therefore technically the freeing of memory can also be done
/// on C side
///
/// # Arguments
///
/// * `examiner_ptr` - Apointer to an initialized wrapped examiner.
pub unsafe extern "C" fn free_wrapped_examiner(examiner_ptr: *mut WrappedExaminer) {
    let _ = Box::from_raw(examiner_ptr);
}
