use std::path::PathBuf;

use clap::{Command, value_parser, arg};
use mistletoe::module::MistHuskModule;

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
        println!("{}", module.generate("name: my-nginx\nnamespace: my-namespace").unwrap());
    }
}