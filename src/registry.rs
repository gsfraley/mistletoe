use crate::config::{MIST_HOME_LOCATION, MistletoeConfig};

use std::path::{Path, PathBuf};

use git2::Repository;

pub struct Registry {
    name: String,
    git: String,
}

impl Registry {
    pub fn from_name(name: &str, config: &MistletoeConfig) -> Option<Self> {
        config.spec.registries.get(name).map(|r| Registry {
            name: name.to_string(),
            git: r.git.clone()
        })
    }

    fn get_local_registry_path(&self) -> PathBuf {
        MIST_HOME_LOCATION
            .join(Path::new("registries"))
            .join(Path::new(&self.name))
    }

    pub fn init(&self) -> anyhow::Result<()> {
        let initted = self.get_local_registry_path()
            .join(Path::new(".git"))
            .exists();

        if !initted {
            std::fs::create_dir_all(self.get_local_registry_path())?;
            Repository::clone(&self.git, self.get_local_registry_path())?;
        }

        Ok(())
    }

    pub fn pull(&self) -> anyhow::Result<()> {
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

    pub fn lookup_package(&self, package: &Path, version: &str) -> Option<PathBuf> {
        let package_path = self.get_local_registry_path()
            .join(&package)
            .join(format!("{}-{}.mist-pack.wasm",
                package.file_name().unwrap().to_str().unwrap(), version));
        
        if package_path.exists() { Some(package_path) } else { None }
    }
}
