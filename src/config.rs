use std::ffi::OsStr;
use std::path::{PathBuf, Path};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use windows::core::PCSTR;
use windows::Win32::Storage::FileSystem::{GetFileAttributesA, SetFileAttributesA, FILE_FLAGS_AND_ATTRIBUTES};

pub static MIST_HOME_LOCATION: Lazy<PathBuf> = Lazy::new(||
    std::env::var("MIST_HOME_LOCATION").map(PathBuf::from).unwrap_or_else(|_|
        home::home_dir().unwrap().join(PathBuf::from(".mistletoe"))));

pub static MIST_CONFIG_LOCATION: Lazy<PathBuf> = Lazy::new(||
    MIST_HOME_LOCATION.join(PathBuf::from("config.yaml")));

const MIST_CONFIG_DEFAULT_CONTENTS: &'static str = include_str!("../res/default_config.yaml");

const API_VERSION: &'static str = "mistletoe.dev/v1alpha1";
const KIND: &'static str = "MistletoeConfig";

// Serde's default attribute references values by function
fn default_api_version() -> String { API_VERSION.to_string() }
fn default_kind() -> String { KIND.to_string() }

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigLayout {
    #[serde(default = "default_api_version")]
    api_version: String,
    #[serde(default = "default_kind")]
    kind: String,
    pub spec: SpecLayout,
}

impl ConfigLayout {
    pub fn new() -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: KIND.to_string(),
            spec: SpecLayout {
                registries: Vec::new(),
            },
        }
    }

    pub fn from_str(config_str: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(config_str)?)
    }

    pub fn from_file(path: &Path) -> anyhow::Result<ConfigLayout> {
        Self::from_str(&std::fs::read_to_string(path)?)
    }

    fn create_env() -> anyhow::Result<()> {
        if !MIST_HOME_LOCATION.is_dir() {
            std::fs::create_dir(&*MIST_HOME_LOCATION)?;

            // This is awful that this is the actual official Rust "windows" crate.
            #[cfg(windows)] {
                unsafe {
                    let mist_home_ptr: *mut OsStr = Box::into_raw(
                        MIST_HOME_LOCATION.canonicalize()?.into_os_string().into_boxed_os_str());
                    let mist_home_pcstr = PCSTR::from_raw(mist_home_ptr as *const u8);

                    let dir_attributes = GetFileAttributesA(mist_home_pcstr);
                    SetFileAttributesA(mist_home_pcstr,
                        FILE_FLAGS_AND_ATTRIBUTES(dir_attributes | 2))?;

                    let _ = Box::from_raw(mist_home_ptr);
                }
            }
        }

        if !MIST_CONFIG_LOCATION.is_file() {
            std::fs::write(&*MIST_CONFIG_LOCATION, MIST_CONFIG_DEFAULT_CONTENTS)?;
        }

        Ok(())
    }

    pub fn from_env() -> anyhow::Result<Self> {
        Self::create_env()?;
        Self::from_file(&*MIST_CONFIG_LOCATION)
    }

    pub fn write_to_file(&self, path: &Path) -> anyhow::Result<()> {
        std::fs::write(path, serde_yaml::to_string(self)?)?;
        Ok(())
    }

    pub fn write_to_env(&self) -> anyhow::Result<()> {
        Self::create_env()?;
        self.write_to_file(&*MIST_CONFIG_LOCATION)?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct SpecLayout {
    pub registries: Vec<RegistryLayout>,
}

impl SpecLayout {
    pub fn lookup_registry(&self, name: &str) -> Option<&RegistryLayout> {
        self.registries.iter()
            .filter(|registry| registry.name == name)
            .next()
    }

    pub fn lookup_registry_mut(&mut self, name: &str) -> Option<&mut RegistryLayout> {
        self.registries.iter_mut()
            .filter(|registry| registry.name == name)
            .next()
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryLayout {
    pub name: String,
    pub default_remote: String,
    pub remotes: Vec<RemoteLayout>,
}

impl RegistryLayout {
    pub fn lookup_remote(&self, name: &str) -> Option<&RemoteLayout> {
        self.remotes.iter()
            .filter(|remote| remote.name() == name)
            .next()
    }

    pub fn lookup_remote_mut(&mut self, name: &str) -> Option<&mut RemoteLayout> {
        self.remotes.iter_mut()
            .filter(|remote| remote.name() == name)
            .next()
    }

    pub fn lookup_default_remote(&self) -> Option<&RemoteLayout> {
        self.lookup_remote(&self.default_remote)
    }

    pub fn lookup_default_remote_mut(&mut self) -> Option<&mut RemoteLayout> {
        self.lookup_remote_mut(&self.default_remote.clone())
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RemoteLayout {
    Git {
        name: String,
        git: GitRemoteLayout,
    },
}

impl RemoteLayout {
    fn name(&self) -> &str {
        match self {
            RemoteLayout::Git { name, git: _ } => name,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GitRemoteLayout {
    pub url: String,
}
