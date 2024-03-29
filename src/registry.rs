use crate::config::{MIST_HOME_LOCATION, RemoteLayout, ConfigLayout};

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use git2::Repository;
use once_cell::sync::Lazy;

pub static MIST_REGISTRIES_LOCATION: Lazy<PathBuf> = Lazy::new(||
    MIST_HOME_LOCATION.join(Path::new("registries")));

pub struct Remote {
    registry_name: String,
    layout: RemoteLayout,
}

impl Remote {
    pub fn new(registry_name: String, layout: RemoteLayout) -> Self {
        Self { registry_name, layout }
    }

    pub fn default_for_name(name: &str, config: &ConfigLayout) -> anyhow::Result<Self> {
        let registry = config.spec.lookup_registry(name)
            .ok_or(anyhow!("could not find registry with the name \"{}\"", name))?;
        let remote = registry.lookup_default_remote()
            .ok_or(anyhow!("registry \"{}\" did not have a remote by the default name \"{}\"", name, registry.default_remote))?;

            
        Ok(Self::new(registry.name.clone(), remote.clone()))
    }

    pub fn init(&self) -> anyhow::Result<()> {
        match &self.layout {
            RemoteLayout::Git { name: _, git }
                => GitRemote { url: git.url.clone() }.init(&self.registry_name),
        }
    }

    pub fn pull(&self) -> anyhow::Result<()> {
        match &self.layout {
            RemoteLayout::Git { name: _, git }
                => GitRemote { url: git.url.clone() }.pull(&self.registry_name),
        }
    }

    pub fn lookup_package(&self, package: &Path, version: &str) -> Option<PathBuf> {
        match &self.layout {
            RemoteLayout::Git { name: _, git }
                => GitRemote { url: git.url.clone() }.lookup_package(&self.registry_name, package, version),
        }
    }
}

struct GitRemote {
    url: String,
}

impl GitRemote {
    fn get_local_registry_path(&self, registry_name: &str) -> PathBuf {
        MIST_REGISTRIES_LOCATION.join(Path::new(registry_name))
    }
    
    fn init(&self, registry_name: &str) -> anyhow::Result<()> {
        let initted = self.get_local_registry_path(registry_name)
            .join(Path::new(".git"))
            .exists();

        if !initted {
            std::fs::create_dir_all(self.get_local_registry_path(registry_name))?;
            Repository::clone(&self.url, self.get_local_registry_path(registry_name))?;
        }

        Ok(())
    }

    fn pull(&self, registry_name: &str) -> anyhow::Result<()> {
        let repository = Repository::open(self.get_local_registry_path(registry_name))?;
        let head = repository.head()?.shorthand().unwrap().to_string();

        repository.find_remote("origin")?
            .fetch(&[&head], None, None)?;

        repository.reset(
            repository.find_branch(&format!("origin/{}", &head), git2::BranchType::Remote)?
                .into_reference().peel_to_commit()?.as_object(),
            git2::ResetType::Hard,
            None)?;

        Ok(())
    }

    fn lookup_package(&self, registry_name: &str, package: &Path, version: &str) -> Option<PathBuf> {
        let package_path = self.get_local_registry_path(registry_name)
            .join(&package)
            .join(format!("{}-{}.mist-pack.wasm",
                package.file_name().unwrap().to_str().unwrap(), version));
        
        if package_path.exists() { Some(package_path) } else { None }
    }
}

pub fn process_registries(config: &ConfigLayout) -> anyhow::Result<()> {
    let found_registries = fs::read_dir(&*MIST_REGISTRIES_LOCATION)?
        .collect::<Result<Vec<fs::DirEntry>, _>>()?.iter()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path().file_name().unwrap().to_str().unwrap().to_string())
        .collect::<Vec<String>>();

    let registries = config.spec.registries.iter()
        .map(|registry| registry.name.clone())
        .collect::<Vec<String>>();

    // Clean all non-declared registries
    for found_registry in found_registries {
        if !registries.contains(&found_registry) {
            fs::remove_dir_all(MIST_REGISTRIES_LOCATION.join(Path::new(&found_registry)))?;
        }
    }

    // Init and pull declared registries
    for registry in &config.spec.registries {
        let remote = Remote::default_for_name(&registry.name, config)?;
        remote.init()?;
        remote.pull()?;
    }

    Ok(())
}
