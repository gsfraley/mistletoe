use std::fs;
use std::path::PathBuf;

use clap::{Command, value_parser, arg};
use mistletoe::module::MistHuskModule;
use mistletoe_api::v0_1::MistHuskInput;

fn main() {
    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .about("Next-level Kubernetes package manager")
        .subcommand(
            Command::new("generate")
                .about("Generate output YAML from a module")
                .arg(arg!([module] "module to call")
                    .required(true)
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(-f --inputfile <FILE> "input file containing values to pass to the module")
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(-s --set <VALUES> "set values to pass to the module")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        let module_path = matches.get_one::<PathBuf>("module").unwrap();
        let mut module = MistHuskModule::from_file(&module_path).unwrap();

        let input_file_yaml = if let Some(input_file) = matches.get_one::<PathBuf>("inputfile") {
            let input_file_string = String::from_utf8(fs::read(input_file).unwrap()).unwrap();
            serde_yaml::from_str::<serde_yaml::Mapping>(&input_file_string).unwrap()
        } else {
            serde_yaml::from_str("{}").unwrap()
        };

        let input_sets_yaml = if let Some(input_sets) = matches.get_one::<String>("set") {
            serde_yaml::from_str::<serde_yaml::Mapping>(&format!("{{{input_sets}}}")).unwrap()
        } else {
            serde_yaml::from_str("{}").unwrap()
        };

        let mut input_mapping = serde_yaml::Mapping::new();
        input_file_yaml.into_iter().for_each(|(key, value)| { input_mapping.insert(key, value); });
        input_sets_yaml.into_iter().for_each(|(key, value)| { input_mapping.insert(key, value); });

        let input = serde_yaml::to_string(&MistHuskInput { data: input_mapping }).unwrap();

        println!("{}", module.generate(&input).unwrap());
    }
}
