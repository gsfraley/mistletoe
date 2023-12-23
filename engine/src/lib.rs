use std::path::Path;
use mistletoe_core::MistOutput;
use wasmer::{Store, Module, Memory, MemoryType, Instance, TypedFunction, imports};

pub fn run_package(path: &Path, input: String) -> anyhow::Result<MistOutput> {
    let mut store = Store::default();
    let module = Module::from_file(&store, path)?;
    let memory = Memory::new(&mut store, MemoryType::new(1, None, false))?;
    let import_object = imports! {};
    let instance = Instance::new(&mut store, &module, &import_object)?;

    let add_to_stack_pointer: TypedFunction<i32, i32>
        = instance.exports.get_typed_function(&mut store, "__wbindgen_add_to_stack_pointer")?;
    let malloc: TypedFunction<(i32, i32), i32>
        = instance.exports.get_typed_function(&mut store, "__wbindgen_malloc")?;
    let process: TypedFunction<(i32, i32, i32), ()>
        = instance.exports.get_typed_function(&mut store, "process")?;
    
    let retptr = add_to_stack_pointer.call(&mut store, -16)?;
    let inptr = malloc.call(&mut store, input.len() as i32, 1)?;
    memory.view(&store).write(inptr.try_into()?, input.as_bytes())?;

    process.call(&mut store, retptr, inptr, input.len() as i32)?;

    let mut outptr_buf: [u8; 4] = [0; 4];
    memory.view(&store).read((retptr/4) as u64, &mut outptr_buf)?;
    let outptr = i32::from_be_bytes(outptr_buf);

    let mut outlen_buf: [u8; 4] = [0; 4];
    memory.view(&store).read((retptr/4 + 1) as u64, &mut outlen_buf)?;
    let outlen = i32::from_be_bytes(outlen_buf);

    let mut output_buf: Vec<u8> = Vec::with_capacity(outlen as usize);
    memory.view(&store).read(outptr as u64, &mut output_buf)?;

    Ok(serde_yaml::from_str(&String::from_utf8(output_buf)?)?)
}