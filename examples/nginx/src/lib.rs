extern crate mistletoe_api;

mod generate;

use std::sync::atomic::AtomicPtr;
use self::generate::generate;

use indoc::concatdoc;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

const INFO: &'static str = concatdoc! {"
    apiVersion: mistletoe.dev/v1alpha1
    kind: MistHuskPackage
    metadata:
      name: example-nginx
      labels:
        mistledoe.dev/group: mistletoe-examples
    spec:
      functions:
        generate: mistletoe_generate
        alloc: mistletoe_alloc
        dealloc: mistletoe_dealloc
"};

static INFO_PTR: Lazy<AtomicPtr<[usize; 2]>> = Lazy::new(|| {
    let wide_ptr = Box::new([INFO.as_ptr() as usize, INFO.len()]);
    AtomicPtr::new(Box::into_raw(wide_ptr))
});

#[wasm_bindgen]
pub fn mistletoe_info() -> *mut [usize; 2] {
    unsafe { *INFO_PTR.as_ptr() }
}

#[wasm_bindgen]
pub fn mistletoe_alloc(len: usize) -> *mut u8 {
    unsafe {
        let layout = std::alloc::Layout::from_size_align(len, std::mem::align_of::<u8>()).unwrap();
        std::alloc::alloc(layout)
    }
}

#[wasm_bindgen]
pub fn mistletoe_dealloc(ptr: *mut u8, len: usize) {
    unsafe {
        let layout = std::alloc::Layout::from_size_align(len, std::mem::align_of::<u8>()).unwrap();
        std::alloc::dealloc(ptr, layout);
    }
}

#[wasm_bindgen]
pub fn mistletoe_generate(ptr: *const u8, len: usize) -> *mut [usize; 2] {
    let input = unsafe { std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap() };
    let mut output = generate(input).into_boxed_str();
    let retptr = Box::into_raw(Box::new([output.as_mut_ptr() as usize, output.len()]));
    std::mem::forget(output);
    retptr
}
