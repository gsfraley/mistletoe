use crate::installation::{InstallResources, InstallRef};
use crate::instance::{MistPackageInstance, MistPackageRef};

use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::ArgMatches;
use mistletoe_api::v1alpha1::MistInput;

pub async fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let package = matches.get_one::<String>("package").unwrap();

    let input_file_yaml = if let Some(input_file) = matches.get_one::<PathBuf>("inputfile") {
        let input_file_string = String::from_utf8(fs::read(input_file)?)?;
        serde_yaml::from_str::<serde_yaml::Mapping>(&input_file_string)?
    } else {
        serde_yaml::from_str("{}")?
    };

    let input_sets_yaml = if let Some(input_sets) = matches.get_one::<String>("set") {
        serde_yaml::from_str::<serde_yaml::Mapping>(&format!("{{{input_sets}}}"))?
    } else {
        serde_yaml::from_str("{}")?
    };

    let mut input_mapping = serde_yaml::Mapping::new();
    input_file_yaml.into_iter().for_each(|(key, value)| { input_mapping.insert(key, value); });
    input_sets_yaml.into_iter().for_each(|(key, value)| { input_mapping.insert(key, value); });

    let name = matches.get_one::<String>("name").ok_or(anyhow!("'name' must be provided"))?;
    input_mapping.insert(serde_yaml::Value::String("name".to_string()), serde_yaml::Value::String(name.clone()));

    let input = serde_yaml::to_string(&MistInput { data: input_mapping })?;
    let mut instance  = MistPackageInstance::load(&MistPackageRef::from_str(&package)?)?;
    let output = instance.generate(&input)?;

    if let Some(message) = output.get_message() {
        println!("{}", message);
    }

    for (_, content) in output.get_files() {
        InstallRef { name: name.to_string(), version: 0}
            .apply_resources(&InstallResources::from_str(content)?
                .label_resources_with(name, 0)?).await?;
    }
    
    Ok(())
}
