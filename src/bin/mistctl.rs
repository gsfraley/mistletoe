use std::path::PathBuf;

use clap::{ArgMatches, Command, arg, value_parser};
use colored::Colorize;
use mistletoe::command::*;

fn main() {
    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .about("Next-level Kubernetes package manager")
        .subcommand(
            Command::new("generate")
                .about("Generate output YAML from a package")
                .arg(arg!([name] "the name of the installation")
                    .required(true))
                .arg(arg!(-p --package <PACKAGE> "package to call")
                    .required(true))
                .arg(arg!(-f --inputfile <FILE> "input file containing values to pass to the package")
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(-s --set <VALUES> "set values to pass to the package"))
                .arg(arg!(-o --output <TYPE> "output type, can be 'yaml', 'raw', or 'dir=<dirpath>'")),
        )
        .subcommand(
            Command::new("inspect")
                .about("Inspects the info exported by a package")
                .arg(arg!([package] "the package to inspect"))
        )
        .get_matches();

    if let Err(e) = run_cli(&matches) {
        eprintln!("{}{} {}", "error".bold().red(), ":".bold(), e.to_string());
    }

}

fn run_cli(matches: &ArgMatches) -> anyhow::Result<()> {
    if let Some(matches) = matches.subcommand_matches("generate") {
        generate::run_command(&matches)?;
    }

    if let Some(matches) = matches.subcommand_matches("inspect") {
        inspect::run_command(&matches)?;
    }

    Ok(())
}
