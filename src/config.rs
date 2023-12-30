use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use windows::Win32::Storage::FileSystem::{
                        GetFileAttributesA, 
                        SetFileAttributesA};
use windows::{Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES, core::PCSTR};

pub static MIST_HOME_LOCATION: Lazy<PathBuf> = Lazy::new(||
    std::env::var("MIST_HOME_LOCATION").map(PathBuf::from).unwrap_or_else(|_|
        home::home_dir().unwrap().join(PathBuf::from(".mistletoe"))));

static MIST_CONFIG_LOCATION: Lazy<PathBuf> = Lazy::new(||
    MIST_HOME_LOCATION.join(PathBuf::from("config.yaml")));

const MIST_CONFIG_DEFAULT_CONTENTS: &'static str = include_str!("../res/default_config.yaml");

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MistletoeConfig {
    pub spec: MistletoeConfigSpec,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MistletoeConfigSpec {
    #[serde(default)]
    pub registries: HashMap<String, MistletoeRegistry>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MistletoeRegistry {
    pub git: String,
}

impl MistletoeConfig {
    pub fn from_str(config: &str) -> anyhow::Result<MistletoeConfig> {
        Ok(serde_yaml::from_str(config)?)
    }

    pub fn from_file(path: &Path) -> anyhow::Result<MistletoeConfig> {
        Self::from_str(&std::fs::read_to_string(path)?)
    }

    pub fn from_env() -> anyhow::Result<MistletoeConfig> {
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

        Self::from_file(&*MIST_CONFIG_LOCATION)
    }
}
