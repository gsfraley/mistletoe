use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use indoc::formatdoc;
use mistletoe_api::v1alpha1::{MistPackage, MistResult, deserialize_result};
use wasmer::{
    Store,
    Module,
    Instance,
    Memory,
    TypedFunction,
    imports,
};

use crate::registry::Registry;

pub struct MistPackageInstance {
    local: bool,
    store: Store,
    instance: Instance,
    package: MistPackage,
}

impl MistPackageInstance {
    pub fn load(target: &str, allow_local: bool) -> anyhow::Result<Self> {
        if PathBuf::from(target).is_absolute() ||
            [
                "/",
                ".",
                std::path::MAIN_SEPARATOR_STR,
                &format!(".{}", std::path::MAIN_SEPARATOR_STR)
            ]
            .iter().any(|p| target.starts_with(p))
        {    
            if !allow_local {
                return Err(anyhow!(formatdoc!{"
                    engine is not permitted to load local module: {}

                    This can happen if a remote reference to a package was run, and that package tries to
                    load a local dependency.  Only local packages can load local packages.",
                    target}));
            }

            let store = Store::default();
            let path = PathBuf::from(&target[2..]);
            let module = Module::from_file(&store, &path)
                .with_context(|| format!("could not find the package at {:?}", &path))?;

            return Ok(Self::init(true, store, module)?);
        }

        let target_parts: Vec<&str> = target.split(":").collect();
        if target_parts.len() < 2 {
            return Err(anyhow!("version must always be specified, in the form `<package>:<version>`"));
        }
        if target_parts.len() > 2 {
            return Err(anyhow!("expected only one ':', found {}", target_parts.len()-1));
        }

        let target_path = Path::new(target_parts.get(0).unwrap());
        let target_version = target_parts.get(1).unwrap();

        let target_registry = target_path.iter().next().unwrap();
        let target_package = target_path.iter()
            .skip(1).map(|p| PathBuf::from(p)).reduce(|p1, p2| p1.join(p2))
            .unwrap();

        let registry = Registry::from_name(
            target_registry.to_str().unwrap(),
            &crate::config::MistletoeConfig::from_env()?);

        if registry.is_none() {
            return Err(anyhow!(
                "could not find a registry saved with the name {}",
                target_registry.to_str().unwrap()));
        }

        let registry = registry.unwrap();
        registry.init()?;
        registry.pull()?;

        let package_path = registry
            .lookup_package(&target_package, target_version)
            .ok_or_else(|| anyhow!("could not find package at {}", target))?;

        Ok(Self::load(package_path.to_str().unwrap(), allow_local)?)
    }

    fn init(local: bool, mut store: Store, module: Module) -> anyhow::Result<Self> {
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;
        let memory = instance.exports.get_memory("memory")?;

        let package = Self::info_from_instance(&mut store, &instance, memory)?;

        Ok(Self {
            local,
            store,
            instance,
            package,
        })
    }

    pub fn is_local(&self) -> bool {
        self.local
    }

    fn info_from_instance(store: &mut Store, instance: &Instance, memory: &Memory)
        -> anyhow::Result<MistPackage>
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
        let info = serde_yaml::from_str(&String::from_utf8(info_buf)?)?;

        Ok(info)
    }

    pub fn info(&mut self) -> anyhow::Result<MistPackage> {
        let memory = (&self.instance.exports).get_memory("memory")?;
        Self::info_from_instance(&mut self.store, &self.instance, memory)
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

    pub fn generate(&mut self, input: &str) -> MistResult {
        let function_generate: TypedFunction<(i32, i32), i32>
            = self.instance.exports.get_typed_function(&mut self.store,
                &self.package.function_generate.clone().unwrap_or("__mistletoe_generate".to_string()))?;

        let input_ptr = self.write_string_to_memory(input)?;
        let output_ptr = function_generate.call(&mut self.store, input_ptr, input.len().try_into()?)?;
        let output = self.read_string_from_memory(output_ptr)?;

        self.dealloc(input_ptr, input.len().try_into()?)?;

        let result = deserialize_result(&output)?;
        Ok(result)
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
