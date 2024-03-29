use mistletoe_api::v1alpha1::MistPackage;

use indexmap::IndexMap;
use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;
use serde_yaml;
use unindent::unindent;

#[derive(Deserialize, Debug)]
struct MistHeaders {
    name: String,
    #[serde(default)]
    labels: Option<IndexMap<String, String>>,
}

/// Generates "headers" for the engine to talk to the package.
/// 
/// This macro takes some package metadata, and outputs some package info as well as some functions to hook
/// into the main entrypoint of your package.  It roughly takes the following:
/// 
/// ```rust
/// mistletoe_package! {"
///   name: namespace-example
///   labels:
///     mistletoe.dev/group: mistletoe-examples
/// "}
/// ```
/// 
/// It turns that into the following API object:
/// 
/// ```yaml
/// apiVersion: mistletoe.dev/v1alpha1
/// kind: MistPackage
/// metadata:
///   name: namespace-example
///   labels:
///     mistletoe.dev/group: mistletoe-examples
/// ```
/// 
/// It also wraps a `pub fn generate` which you must provide.
/// 
/// It generates the following hooks for the outer runtime to call:
/// 
/// ```txt
/// __mistletoe_info: [] -> [I32] // Returns the MistPackage API object
/// __mistletoe_alloc: [I32] -> [I32] // Passthrough to rust allocator
/// __mistletoe_dealloc: [I32, I32] -> [] // Passthrough to rust (de)allocator
/// __mistletoe_generate: [I32, I32] -> [I32] // Wrapper around your pub generate function
/// ```
#[proc_macro]
pub fn mistletoe_package(input: TokenStream) -> TokenStream {
    let header_string_unfmt = input.into_iter().next().unwrap().to_string();
    let header_string = unindent(&header_string_unfmt[1..header_string_unfmt.len()-1]);
    let headers: MistHeaders = serde_yaml::from_str(&header_string).unwrap();

    let mistpackage = MistPackage {
        name: headers.name,
        labels: headers.labels,
    };

    let mistpackage_string = serde_yaml::to_string(&mistpackage).unwrap();

    quote! {
        const INFO: &'static str = #mistpackage_string;
        
        static INFO_PTR: mistletoe_bind::include::once_cell::sync::Lazy<std::sync::atomic::AtomicPtr<[usize; 2]>>
            = mistletoe_bind::include::once_cell::sync::Lazy::new(||
        {
            let wide_ptr = Box::new([INFO.as_ptr() as usize, INFO.len()]);
            std::sync::atomic::AtomicPtr::new(Box::into_raw(wide_ptr))
        });
        
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn __mistletoe_info() -> *mut [usize; 2] {
            unsafe { *INFO_PTR.as_ptr() }
        }
        
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn __mistletoe_alloc(len: usize) -> *mut u8 {
            unsafe {
                let layout = std::alloc::Layout::from_size_align(len, std::mem::align_of::<u8>()).unwrap();
                std::alloc::alloc(layout)
            }
        }
        
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn __mistletoe_dealloc(ptr: *mut u8, len: usize) {
            unsafe {
                let layout = std::alloc::Layout::from_size_align(len, std::mem::align_of::<u8>()).unwrap();
                std::alloc::dealloc(ptr, layout);
            }
        }

        fn __mistletoe_generate_result(input_str: &str) -> mistletoe_api::v1alpha1::MistResult {
            let input: mistletoe_api::v1alpha1::MistInput = mistletoe_bind::include::serde_yaml::from_str(input_str)?;
            generate(mistletoe_api::v1alpha1::yaml_transmute(input)?)
        }
        
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn __mistletoe_generate(ptr: *const u8, len: usize) -> *mut [usize; 2] {
            let input_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap() };
            let result = __mistletoe_generate_result(input_str);
            let mut output_str = std::mem::ManuallyDrop::new(mistletoe_api::v1alpha1::serialize_result(&result).unwrap());
            let retptr = Box::into_raw(Box::new([output_str.as_mut_ptr() as usize, output_str.len()]));
            retptr
        }
    }.into()
}
