use crate::config::MistletoeConfig;
use crate::instance::MistPackageInstance;
use crate::registry::Registry;

use std::path::{Path, PathBuf};

use anyhow::anyhow;
use wasmer::{Module, Store};


pub enum PackageRef {
    Local(PathBuf),
    Remote {
        registry: String,
        package: String,
        version: String,
    },
}

pub fn load(package_ref: &PackageRef) -> anyhow::Result<MistPackageInstance> {
    let store = Store::default();

    match package_ref {
        PackageRef::Local(package_path) => {
            let module = Module::from_file(&store, package_path)?;
            Ok(MistPackageInstance::init(true, store, module)?)
        },
        PackageRef::Remote { registry, package, version } => {
            let registry_instance = Registry::from_name(
                registry,
                &MistletoeConfig::from_env()?);

            if registry_instance.is_none() {
                return Err(anyhow!(
                    "could not find a registry saved with the name {}",
                    registry));
            }

            let registry_instance = registry_instance.unwrap();
            registry_instance.init()?;
            registry_instance.pull()?;

            let package_path = registry_instance
                .lookup_package(&PathBuf::from(package), version)
                .ok_or_else(|| anyhow!("could not find package at {}", package))?;

            let module = Module::from_file(&store, package_path)?;
            Ok(MistPackageInstance::init(true, store, module)?)
        }
    }
}

impl PackageRef {
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
