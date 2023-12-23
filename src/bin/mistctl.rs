extern crate mistletoe_api;

use std::path::Path;
use wasmer::{Store, Module, Instance, TypedFunction, imports};

pub fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let module_path = Path::new(&args[1]);
    let _input_path = Path::new(&args[2]);
    let input = "".to_string(); //String::from_utf8(std::fs::read(input_path)?)?;
    run_package(module_path, input)?;

    Ok(())
}

pub fn run_package(path: &Path, _input: String) -> anyhow::Result<()> {
    let mut store = Store::default();
    let module = Module::from_file(&store, path)?;
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;
    let memory = instance.exports.get_memory("memory")?;

    let mistletoe_info: TypedFunction<(), i32>
        = instance.exports.get_typed_function(&mut store, "mistletoe_info")?;
    let info_ptr_ptr = mistletoe_info.call(&mut store)?;
    let mut info_ptr_buf: [u8; 8] = [0; 8];
    memory.view(&mut store).read(info_ptr_ptr as u64, &mut info_ptr_buf)?;
    let info_ptr = i32::from_le_bytes(info_ptr_buf[0..4].try_into()?);
    let info_len = i32::from_le_bytes(info_ptr_buf[4..8].try_into()?);
    let mut info_buf: Vec<u8> = vec![0; info_len as usize];
    memory.view(&mut store).read(info_ptr as u64, &mut info_buf[..])?;
    let info = String::from_utf8(info_buf)?;
    println!("{}", info);

    Ok(())
}
