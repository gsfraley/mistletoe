use mistletoe_api::v0_1::MistHuskPackage;

use indexmap::IndexMap;
use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;
use serde_yaml;
use unindent::unindent;

#[derive(Deserialize, Debug)]
struct MistHuskHeaders {
    name: String,
    #[serde(default)]
    labels: Option<IndexMap<String, String>>,
}

#[proc_macro]
pub fn misthusk_headers(input: TokenStream) -> TokenStream {
    let header_string_unfmt = input.into_iter().next().unwrap().to_string();
    let header_string = unindent(&header_string_unfmt[1..header_string_unfmt.len()-1]);
    let headers: MistHuskHeaders = serde_yaml::from_str(&header_string).unwrap();

    let misthuskpackage = MistHuskPackage {
        name: headers.name,
        labels: headers.labels,

        function_generate: Some("__mistletoe_generate".to_string()),
        function_alloc: Some("__mistletoe_alloc".to_string()),
        function_dealloc: Some("__mistletoe_dealloc".to_string()),
    };

    let misthuskpackage_string = serde_yaml::to_string(&misthuskpackage).unwrap();

    quote! {
        const INFO: &'static str = #misthuskpackage_string;
        
        static INFO_PTR: Lazy<AtomicPtr<[usize; 2]>> = Lazy::new(|| {
            let wide_ptr = Box::new([INFO.as_ptr() as usize, INFO.len()]);
            AtomicPtr::new(Box::into_raw(wide_ptr))
        });
        
        #[wasm_bindgen]
        pub fn __mistletoe_info() -> *mut [usize; 2] {
            unsafe { *INFO_PTR.as_ptr() }
        }
        
        #[wasm_bindgen]
        pub fn __mistletoe_alloc(len: usize) -> *mut u8 {
            unsafe {
                let layout = std::alloc::Layout::from_size_align(len, std::mem::align_of::<u8>()).unwrap();
                std::alloc::alloc(layout)
            }
        }
        
        #[wasm_bindgen]
        pub fn __mistletoe_dealloc(ptr: *mut u8, len: usize) {
            unsafe {
                let layout = std::alloc::Layout::from_size_align(len, std::mem::align_of::<u8>()).unwrap();
                std::alloc::dealloc(ptr, layout);
            }
        }
        
        #[wasm_bindgen]
        pub fn __mistletoe_generate(ptr: *const u8, len: usize) -> *mut [usize; 2] {
            let input = unsafe { std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap() };
            let mut output = generate(input).into_boxed_str();
            let retptr = Box::into_raw(Box::new([output.as_mut_ptr() as usize, output.len()]));
            std::mem::forget(output);
            retptr
        }
    }.into()
}
