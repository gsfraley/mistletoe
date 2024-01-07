use crate::config::{MIST_HOME_LOCATION, RemoteLayout, ConfigLayout};

use std::path::{Path, PathBuf};

use anyhow::anyhow;
use git2::Repository;

pub struct Remote {
    layout: RemoteLayout,
}

impl From<RemoteLayout> for Remote {
    fn from(layout: RemoteLayout) -> Self {
        Self { layout }
    }
}

impl Remote {
    pub fn default_for_name(name: &str, config: &ConfigLayout) -> anyhow::Result<Self> {
        let registry = config.spec.lookup_registry(name)
            .ok_or(anyhow!("could not find registry for name \"{}\"", name))?;
        let remote = registry.lookup_default_remote()
            .ok_or(anyhow!("registry \"{}\" did not have a remote by the default name \"{}\"", name, registry.default_remote))?;

        Ok(remote.clone().into())
    }

    pub fn init(&self) -> anyhow::Result<()> {
        match &self.layout {
            RemoteLayout::Git { name, git }
                => GitRemote { name: name.to_string(), url: git.url.clone() }.init(),
        }
    }

    pub fn pull(&self) -> anyhow::Result<()> {
        match &self.layout {
            RemoteLayout::Git { name, git }
                => GitRemote { name: name.to_string(), url: git.url.clone() }.pull(),
        }
    }

    pub fn lookup_package(&self, package: &Path, version: &str) -> Option<PathBuf> {
        match &self.layout {
            RemoteLayout::Git { name, git }
                => GitRemote { name: name.to_string(), url: git.url.clone() }.lookup_package(package, version),
        }
    }
}

struct GitRemote {
    name: String,
    url: String,
}

impl GitRemote {
    fn get_local_registry_path(&self) -> PathBuf {
        MIST_HOME_LOCATION
            .join(Path::new("registries"))
            .join(Path::new(&self.name))
    }
    
    fn init(&self) -> anyhow::Result<()> {
        let initted = self.get_local_registry_path()
            .join(Path::new(".git"))
            .exists();

        if !initted {
            std::fs::create_dir_all(self.get_local_registry_path())?;
            Repository::clone(&self.url, self.get_local_registry_path())?;
        }

        Ok(())
    }

    fn pull(&self) -> anyhow::Result<()> {
        let repository = Repository::open(self.get_local_registry_path())?;
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

    fn lookup_package(&self, package: &Path, version: &str) -> Option<PathBuf> {
        let package_path = self.get_local_registry_path()
            .join(&package)
            .join(format!("{}-{}.mist-pack.wasm",
                package.file_name().unwrap().to_str().unwrap(), version));
        
        if package_path.exists() { Some(package_path) } else { None }
    }
}
