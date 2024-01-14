use crate::installation::InstallResources;

use std::path::{Path, PathBuf};

use anyhow::anyhow;
use kube::core::DynamicObject;
use mistletoe_api::v1alpha1::{MistResult, serialize_result};

pub trait McOutputRaw {
    fn mc_output_raw(self) -> anyhow::Result<String>;
}

pub trait McOutputYaml {
    fn mc_output_yaml(self) -> anyhow::Result<String>;

    fn mc_output_processed_yaml(self, name: String, version: Option<u32>)
        -> anyhow::Result<String>
        where Self : Sized
    {
        InstallResources::from_str(&self.mc_output_yaml()?)?
            .label_resources_with(&name, version)?
            .to_string()
    }
}

pub trait McOutputList {
    fn mc_output_list(self) -> anyhow::Result<String>;
}

pub trait McOutputDir {
    fn mc_output_dir(self, path: &Path) -> anyhow::Result<()>;
}

impl McOutputYaml for &Vec<DynamicObject> {
    fn mc_output_yaml(self) -> anyhow::Result<String> {
        Ok(self.iter()
            .map(serde_yaml::to_string)
            .collect::<Result<Vec<String>, _>>()?
            .join("\n---\n"))
    }
}

impl McOutputList for &Vec<DynamicObject> {
    fn mc_output_list(self) -> anyhow::Result<String> {
        Ok(self.iter()
            .map(|resource| {
                let types = resource.types.clone()
                    .ok_or(anyhow!("resource has no type information"))?;
                let metadata = resource.metadata.clone();

                let kind = types.kind;
                let version_parts: Vec<&str> = types.api_version.split("/").collect();
                let full_type = if version_parts.len() > 1 {
                    format!("{}.{}", kind.to_lowercase(), version_parts[1])
                } else {
                    format!("{}", kind.to_lowercase())
                };

                let name = metadata.name;
                let namespace = metadata.namespace;

                Ok(format!("{}/{} ({})",
                    full_type,
                    name
                        .unwrap_or("<unknown name>".to_string()),
                    namespace.map(|ns| format!("in namespace {}", ns))
                        .unwrap_or("not namespaced".to_string())))
            })
            .collect::<anyhow::Result<Vec<String>>>()?
            .join("\n"))
    }
}

impl McOutputRaw for MistResult {
    fn mc_output_raw(self) -> anyhow::Result<String> {
        Ok(serialize_result(&self)?.trim().to_string())
    }
}

impl McOutputYaml for MistResult {
    fn mc_output_yaml(self) -> anyhow::Result<String> {
        Ok(self?.get_files().iter().map(|(_, content)| content.to_string())
            .collect::<Vec<String>>()
            .join("\n---\n"))
    }
}

impl McOutputList for MistResult {
    fn mc_output_list(self) -> anyhow::Result<String> {
        let objs: Vec<DynamicObject> = serde_yaml::from_str(&self.mc_output_yaml()?)?;
        Ok(objs.mc_output_list()?)
    }
}

impl McOutputDir for MistResult {
    fn mc_output_dir(self, path: &Path) -> anyhow::Result<()> {
        for (filename, content) in self?.get_files() {
            let out_path = path.join(PathBuf::from(filename));

            let parent_dir = out_path.parent()
                .ok_or(anyhow!("really unexpected error, output file path \"{}\" has no parent directory",
                    out_path.display()))?;

            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir)?;
            }

            std::fs::write(out_path, content)?;
        }

        Ok(())
    }
}
