use std::path::{Path, PathBuf};

use anyhow::anyhow;
use git2::Repository;
use semver::Version;
use tempfile::TempDir;

const package_suffix: &'static str = ".mist";

pub struct Registry {
    _tempdir: Option<TempDir>,
    path: PathBuf,
}

impl Registry {
    pub fn from_local_path(path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            _tempdir: None,
            path: path.canonicalize()?,
        })
    }

    pub fn from_git_url(url: &str) -> anyhow::Result<Self> {
        let tempdir = tempfile::tempdir()?;
        let local_path = tempdir.path().to_path_buf();
        Repository::clone(url, &local_path)?;

        Ok(Self {
            _tempdir: Some(tempdir),
            path: local_path.canonicalize()?,
        })
    }

    pub fn get_package_path(&self, package: &str) -> anyhow::Result<PathBuf> {
        let mut package_parts = package.split(":");
        let package_dir = Path::new(package_parts.next().unwrap());
        let package_version = package_parts.next();

        todo!();

        //let package_path = true;
        //let target_path = package_dir.join();

        //if !target_path.starts_with(&self.path) {
        //    Err(anyhow!("package location ({:?}) is outside of registry path ({:?}", package, self.path))?
        //}

        //Ok(target_path)
    }

    pub fn get_latest_version(&self, package: &str) -> anyhow::Result<Version> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    #[test]
    fn test_path_ok() {
        let tempdir = tempfile::tempdir().unwrap();
        let registry = Registry::from_local_path(tempdir.path()).unwrap();

        let expected_package_path = tempdir.path().join("examples/basic-nginx");
        fs::create_dir_all(&expected_package_path).unwrap();
        let expected_package_path = expected_package_path.canonicalize().unwrap();

        let package_path = registry.get_package_path("examples/basic-nginx").unwrap();
        assert_eq!(expected_package_path, package_path);
    }

    #[test]
    fn test_path_safety() {
        let tempdir = tempfile::tempdir().unwrap();
        let registry = Registry::from_local_path(tempdir.path()).unwrap();

        let relative_up = registry.get_package_path("../..").map(|p| p.canonicalize().unwrap());
        assert!(relative_up.is_err(),
            "should not be able to ascend out with '..', result is '{:?}'", relative_up);

        let absolute_up = registry.get_package_path("/dev").map(|p| p.canonicalize().unwrap());
        assert!(absolute_up.is_err(),
            "should not be able to ascent out with '/', result is '{:?}'", absolute_up);
    }
}
