use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Command, ArgMatches, value_parser, arg};
use mistletoe::{OutputMode, output_result};
use mistletoe::husk::MistHuskPackageModule;
use mistletoe_api::v0_1::MistHuskInput;

fn main() {
    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .about("Next-level Kubernetes package manager")
        .subcommand(
            Command::new("generate")
                .about("Generate output YAML from a module")
                .arg(arg!(-p --package <PACKAGE> "package to call")
                    .required(true))
                .arg(arg!(-f --inputfile <FILE> "input file containing values to pass to the module")
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(-s --set <VALUES> "set values to pass to the module"))
                .arg(arg!(-o --output <TYPE> "output type, can be 'yaml', 'raw', or 'dir=<dirpath>'"))
                .arg(arg!(--debug "whether to give additional debug output if applicable")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        if let Err(e) = run_command(&matches) {
            if matches.get_flag("debug") {
                eprintln!("Error: {:?}", e);
            } else {
                eprintln!("Error: {}", e);
            }
        };
    }
}

fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
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

    let output_mode = match matches.get_one::<String>("output").map(|o| o.as_str()) {
        None | Some("yaml") => OutputMode::Yaml,
        Some("raw") => OutputMode::Raw,
        Some(o) if o.starts_with("dir=") => OutputMode::Dir(PathBuf::from(&o[4..])),
        Some(o) => Err(anyhow!("Unexpected output type: {}", o))?,
    };

    let input = serde_yaml::to_string(&MistHuskInput { data: input_mapping })?;
    let mut module = MistHuskPackageModule::load(&package, true)?;
    let result = module.generate(&input);
    
    output_result(result, output_mode)?;

    Ok(())
}
