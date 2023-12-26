use std::path::Path;

use mistletoe_api::v0_1::MistHuskPackage;
use wasmer::{
    Store,
    Module,
    Instance,
    Memory,
    TypedFunction,
    imports,
};

pub struct MistHuskModule {
    store: Store,
    instance: Instance,
    package: MistHuskPackage,
}

impl MistHuskModule {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let mut store = Store::default();
        let module = Module::from_file(&store, path)?;
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;
        let memory = instance.exports.get_memory("memory")?;

        let info = Self::info_from_instance(&mut store, &instance, memory)?;
        let package: MistHuskPackage = serde_yaml::from_str(&info)?;

        Ok(Self {
            store,
            instance,
            package,
        })
    }

    fn info_from_instance(store: &mut Store, instance: &Instance, memory: &Memory)
        -> anyhow::Result<String>
    {
        let function_info: TypedFunction<(), i32>
            = instance.exports.get_typed_function(store, "__mistletoe_info")?;

        let info_ptr_ptr = function_info.call(store)?;
        let mut info_ptr_buf: [u8; 8] = [0; 8];
        memory.view(store).read(info_ptr_ptr as u64, &mut info_ptr_buf)?;
        let info_ptr = i32::from_le_bytes(info_ptr_buf[0..4].try_into()?);
        let info_len = i32::from_le_bytes(info_ptr_buf[4..8].try_into()?);
        let mut info_buf: Vec<u8> = vec![0; info_len.try_into()?];

        memory.view(store).read(info_ptr as u64, &mut info_buf[..])?;
        Ok(String::from_utf8(info_buf)?)
    }

    fn alloc(&mut self, len: i32) -> anyhow::Result<i32> {
        let function_alloc: TypedFunction<i32, i32>
            = self.instance.exports.get_typed_function(&mut self.store,
                &self.package.function_alloc.clone().unwrap_or("__mistletoe_alloc".to_string()))?;
        
        Ok(function_alloc.call(&mut self.store, len)?)
    }

    fn dealloc(&mut self, ptr: i32, len: i32) -> anyhow::Result<()> {
        let function_dealloc: TypedFunction<(i32, i32), ()>
            = self.instance.exports.get_typed_function(&mut self.store,
                &self.package.function_dealloc.clone().unwrap_or("__mistletoe_dealloc".to_string()))?;
        
        function_dealloc.call(&mut self.store, ptr, len)?;
        Ok(())
    }

    pub fn generate(&mut self, input: &str) -> anyhow::Result<String> {
        let function_generate: TypedFunction<(i32, i32), i32>
            = self.instance.exports.get_typed_function(&mut self.store,
                &self.package.function_generate.clone().unwrap_or("__mistletoe_generate".to_string()))?;

        let input_ptr = self.write_string_to_memory(input)?;
        let output_ptr = function_generate.call(&mut self.store, input_ptr, input.len().try_into()?)?;
        let output = self.read_string_from_memory(output_ptr)?;

        self.dealloc(input_ptr, input.len().try_into()?)?;
        Ok(output)
    }

    fn write_string_to_memory(&mut self, input: &str) -> anyhow::Result<i32> {
        let ptr = self.alloc(input.len().try_into()?)?;
        let memory = self.instance.exports.get_memory("memory")?;
        memory.view(&mut self.store).write(ptr as u64, input.as_bytes())?;
        Ok(ptr)
    }

    fn read_string_from_memory(&mut self, ptr: i32) -> anyhow::Result<String> {
        let memory = self.instance.exports.get_memory("memory")?;
        let mut output_ptr_buf: [u8; 8] = [0; 8];
        memory.view(&mut self.store).read(ptr as u64, &mut output_ptr_buf)?;

        let output_ptr = i32::from_le_bytes(output_ptr_buf[0..4].try_into()?);
        let output_len = i32::from_le_bytes(output_ptr_buf[4..8].try_into()?);
        let mut output_buf: Vec<u8> = vec![0; output_len.try_into()?];
        memory.view(&mut self.store).read(output_ptr as u64, &mut output_buf[..])?;
        let output = String::from_utf8(output_buf)?;

        self.dealloc(output_ptr, output_len)?;
        Ok(output)
    }
}
