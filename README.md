# README
## What is this package?
This crate is the rust part of the Poseidon project. The description of which shall be included in that repo's README. 
This crate is to be compiled into a static library and interfaces with C (which serves as an entry point) through C - Rust FFI. 
The main responsibilities for this crate are to:
- Handle incoming interrupts from C side
- Invoke an api provided by C side (if the conditions are right) to turn on the water pump
- Provide an api for C side to call to get the latest humidity reading (this would be slower than just reading a variable stored in C side but it's good for validation purposes so that's what we would do for now).

## High Level Learning Points
Implementing the packages, here are some of the things I learned:

### Self-referential data structure
Source: 
	- page 182 in "Rust for Rustaceans"
	- [Pin module documentation](https://doc.rust-lang.org/std/pin/)

The snippet below does the following:
	- instantiates a Self with a field that is dangling (this is done via `NonNull`, more on that [[Rust Part aka the examiner#^f71ddf|later]] but well aligned (uninitialiazed). We leave this field uninitialized because this field is going to contain self-referential data. 
	- wraps self in Pin with `Box::pin`. We want to do this because we have fields that references other fields in the same struct. If we do not ensure this entire struct has a stable location in memory, all of these references risk being invalidated. 
	- in an unsafe block, it gets the raw pointer from the pin wrapper and assigns itself with the self-referential field
```rust
fn new(data: String) -> Pin<Box<Self>> {
	let res = Unmovable {
		data,
		// we only create the pointer once the data is in place
		// otherwise it will have already moved before we even started
		slice: NonNull::dangling(),
		_pin: PhantomPinned,
	};
	let mut boxed = Box::pin(res);

	let slice = NonNull::from(&boxed.data);
	// we know this is safe because modifying a field doesn't move the whole struct
	unsafe {
		let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
		Pin::get_unchecked_mut(mut_ref).slice = slice;
	}
	boxed
}
```

*NonNull (vs raw pointer)*
	- https://doc.rust-lang.org/std/ptr/struct.NonNull.html
	- You can think of it like a raw pointer, only it is not allowed to be null
	- [dangling](https://doc.rust-lang.org/std/ptr/struct.NonNull.html#method.dangling): think of this as a placeholder for a valid pointer. It is up to the implementor to ensure there is a valid reference here. It isn't null but it's not valid either until it's been properly initialized. 

### no_std
We're going to have to use no_std here because otherwise the static library might be too big. 
In order to still take advantage of the goodies from std, we would have to use alloc: https://doc.rust-lang.org/std/alloc/index.html

> This library provides smart pointers and collections for managing heap-allocated values. 
> This library, like libcore, normally doesn't need to be used directly, since its contents are re-exported in the ['std' crate]. Crates that use `#![no_std]` attributes however will typically not depend on `std`, so they would need to use this crate instead. 

You would need to also declare the use of alloc crate as such:
```Rust
extern crate alloc;
```

*build target*
	- the build target of interest for us is `thumbv6m-none-eabi`
	- to properly convey this message to the IDE and the build system, add the [project config file](https://doc.rust-lang.org/cargo/reference/config.html) at `${project_folder}/.cargo/config.toml` the line `[build]\ntarget = "thumbv6m-none-eabi"`
	- you can check if you have this target installed via running `rustup target list | grep thumbv6m-none-eabi`

*conditional attribute*
	- should you include `no_std`, you would need to also configure your project in such a way that it allows testing.
	- some tests would depend on things that are in `std`. If you simply have `#![no_std]`, your tests would fail. [See this stackoverflow thread here](https://stackoverflow.com/questions/28185854/how-do-i-test-crates-with-no-std).
	- the answer is to use [conditional attribute](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute) to conditionally include the no_std directive.
	- but even all of this doesn't work, see [this thread here](https://github.com/rust-lang/rust/issues/100766)

*testing embedded system code with no_std*
	- https://ferrous-systems.com/blog/test-embedded-app/
	- for this project for some reason following that set up did not work but the following is how I got it to work
		- take out the project level config. This does mean you would need to specify your build target when you build
		- add `#![cfg_attr(not(test), no_std)]` at the root module
		- above where `#[panic_handler]` is, add `#[cfg(not(test))]`, so it does not get compiled for tests. And because when you would need to specify target when you build, the compiler won't complain about this
	- to build, run `cargo +nightly build --release --target thumbv6m-none-eabi`

*alloc*
	- because you are not including `std` , you need to specify the allocator
	- https://stackoverflow.com/questions/74012369/no-global-memory-allocator-found-but-one-is-required-link-to-std-or-add-glob
	- this is still unverified (if it works) but I had deviced externing `free` and `alloc` as such in the following snippet
	- i did have to resort to using +nightly otherwise the toolchain would complain about unstable feature
	-  *remember that for nightly the targets are installed separately so you would need to install thumbv6m-none-eabi again*

```rust
#![feature(default_alloc_error_handler)]
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

/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

```

