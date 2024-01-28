extern "C" {
    pub fn __mistletoe_get_random_bytes(len: usize) -> *mut u8;
}

pub fn get_random_bytes(len: usize) -> Box<[u8]> {
    unsafe {
        let ptr = __mistletoe_get_random_bytes(len);
        Box::from_raw(std::slice::from_raw_parts_mut(ptr, len))
    }
}
