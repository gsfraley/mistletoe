use crate::instance::{MistPackageInstance, MistPackageRef};

use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::ArgMatches;
use mistletoe_api::v1alpha1::{MistInput, MistResult, serialize_result};

pub fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
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

    let output_mode = match matches.get_one::<String>("output").map(|o| o.as_str()) {
        None | Some("yaml") => OutputMode::Yaml,
        Some("raw") => OutputMode::Raw,
        Some(o) if o.starts_with("dir=") => OutputMode::Dir(PathBuf::from(&o[4..])),
        Some(o) => Err(anyhow!("Unexpected output type: {}", o))?,
    };

    let input = serde_yaml::to_string(&MistInput { data: input_mapping })?;
    let mut instance  = MistPackageInstance::load(&MistPackageRef::from_str(&package)?)?;
    let result = instance.generate(&input);
    
    output_result(result, output_mode)?;

    Ok(())
}

enum OutputMode {
    Raw,
    Yaml,
    Dir(PathBuf),
}

fn output_result(result: MistResult, mode: OutputMode) -> anyhow::Result<()> {
    if let Ok(output) = &result {
        if let Some(message) = output.get_message() {
            println!("{}", message);
        }
    }

    match mode {
        OutputMode::Raw => Ok(println!("{}", serialize_result(result)?.trim())),

        OutputMode::Yaml => match result {
            Ok(output) => {
                output.get_files().values()
                    .for_each(|content| println!("{}", content.trim()));

                Ok(())
            },
            Err(e) => Err(e),
        },

        OutputMode::Dir(path) => match result {
            Ok(output) => {
                for (filename, content) in output.get_files() {
                    let out_path = path.join(PathBuf::from(filename));
                    fs::write(out_path, content)?;
                }

                Ok(())
            },
            Err(e) => Err(e),
        },
    }
}
