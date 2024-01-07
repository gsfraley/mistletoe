use crate::config::ConfigLayout;
use crate::registry::Remote;

use std::path::{Path, PathBuf};

use anyhow::anyhow;
use mistletoe_api::v1alpha1::{MistPackage, MistResult, deserialize_result};
use wasmer::{
    Store,
    Module,
    Instance,
    Memory,
    TypedFunction,
    imports,
};


pub enum MistPackageRef {
    Local(PathBuf),
    Remote {
        registry: String,
        package: String,
        version: String,
    },
}

impl MistPackageRef {
    pub fn from_str(package: &str) -> anyhow::Result<Self> {
        let package_path = PathBuf::from(package);

        if package_path.is_absolute() ||
            [
                "/",
                ".",
                std::path::MAIN_SEPARATOR_STR,
            ]
            .iter().any(|p| package.starts_with(p))
        {
            return Ok(Self::Local(package_path));
        }

        let package_parts: Vec<&str> = package.split(":").collect();
        if package_parts.len() < 2 {
            return Err(anyhow!("version must always be specified, in the form `<package>:<version>`"));
        }
        if package_parts.len() > 2 {
            return Err(anyhow!("expected only one ':', found {}", package_parts.len()-1));
        }

        let remote_path = Path::new(package_parts.get(0).unwrap());
        let remote_version = package_parts.get(1).unwrap();

        let remote_registry = remote_path.iter().next().unwrap();
        let remote_package = remote_path.iter()
            .skip(1).map(|p| PathBuf::from(p)).reduce(|p1, p2| p1.join(p2))
            .unwrap();

        Ok(Self::Remote {
            registry: remote_registry.to_str().unwrap().to_string(),
            package: remote_package.to_str().unwrap().to_string(),
            version: remote_version.to_string(),
        })
    }
}

pub struct MistPackageInstance {
    local: bool,
    store: Store,
    instance: Instance,
}

impl MistPackageInstance {
    pub fn load(package_ref: &MistPackageRef) -> anyhow::Result<Self> {
        let store = Store::default();

        match package_ref {
            MistPackageRef::Local(package_path) => {
                let module = Module::from_file(&store, package_path)?;
                Ok(MistPackageInstance::init(true, store, module)?)
            },
            MistPackageRef::Remote { registry, package, version } => {
                let remote = Remote::default_for_name(
                    registry,
                    &ConfigLayout::from_env()?)?;

                remote.init()?;
                remote.pull()?;

                let package_path = remote
                    .lookup_package(&PathBuf::from(package), version)
                    .ok_or_else(|| anyhow!("could not find package at {}", package))?;

                let module = Module::from_file(&store, package_path)?;
                Ok(MistPackageInstance::init(true, store, module)?)
            }
        }
    }

    fn init(local: bool, mut store: Store, module: Module) -> anyhow::Result<Self> {
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;

        Ok(Self {
            local,
            store,
            instance,
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
            = self.instance.exports.get_typed_function(&mut self.store, "__mistletoe_alloc")?;
        
        Ok(function_alloc.call(&mut self.store, len)?)
    }

    fn dealloc(&mut self, ptr: i32, len: i32) -> anyhow::Result<()> {
        let function_dealloc: TypedFunction<(i32, i32), ()>
            = self.instance.exports.get_typed_function(&mut self.store, "__mistletoe_dealloc")?;
        
        function_dealloc.call(&mut self.store, ptr, len)?;
        Ok(())
    }

    pub fn generate(&mut self, input: &str) -> MistResult {
        let function_generate: TypedFunction<(i32, i32), i32>
            = self.instance.exports.get_typed_function(&mut self.store, "__mistletoe_generate")?;

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
