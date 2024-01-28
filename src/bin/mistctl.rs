use std::path::PathBuf;

use clap::{ArgMatches, Command, arg, value_parser};
use colored::Colorize;
use mistletoe::command::*;

#[tokio::main]
async fn main() {
    let matches = Command::new(env!("CARGO_CRATE_NAME"))
        .about("Polyglot Kubernetes package manager")
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
                .arg(arg!(-o --output <TYPE> "output type, can be 'yaml', 'raw', or 'dir=<dirpath>'"))
                .arg(arg!(-r --process "run the processing to set installation labels (will reformat the output YAML)"))
        )
        .subcommand(
            Command::new("install")
                .about("Install a package to the cluster")
                .arg(arg!([name] "the name of the installation")
                    .required(true))
                .arg(arg!(-p --package <PACKAGE> "package to call")
                    .required(true))
                .arg(arg!(-f --inputfile <FILE> "input file containing values to pass to the package")
                    .value_parser(value_parser!(PathBuf)))
                .arg(arg!(-o --output <TYPE> "output type, can be 'details' or 'yaml'"))
                .arg(arg!(-s --set <VALUES> "set values to pass to the package"))
        )
        .subcommand(
            Command::new("uninstall")
                .about("Uninstall a package from the cluster")
                .arg(arg!([name] "the name of the installation")
                    .required(true))
        )
        .subcommand(
            Command::new("inspect")
                .about("Inspects things around Mistletoe and the cluster")
                .subcommand(
                    Command::new("package")
                        .about("Inspects the given package")
                        .arg(arg!([package] "the package to inspect")
                            .required(true))
                )
                .subcommand(
                    Command::new("install")
                        .about("Inspects the given cluster installation")
                        .arg(arg!([name] "the name of the installation")
                            .required(true))
                        .arg(arg!(-o --output <TYPE> "output type, can be 'yaml' or 'list'"))
                )
        )
        .subcommand(
            Command::new("registry")
                .about("Manage the configured registries for Mistletoe")
                .subcommand(
                    Command::new("add")
                        .about("Adds a new registry")
                        .arg(arg!([name] "the name to give the registry")
                            .required(true))
                        .arg(arg!(-g --git <URL> "a git remote url"))
                )
                .subcommand(
                    Command::new("list")
                        .about("Lists the configured registries")
                )
                .subcommand(
                    Command::new("remove")
                        .about("Removes the given registry")
                        .arg(arg!([name] "the name of the registry to remove")
                            .required(true))
                )
        )
        .arg(arg!(-d --debug "enable debug output")
            .global(true))
        .get_matches();

    if let Err(e) = run_cli(&matches).await {
        if matches.get_flag("debug") {
            eprintln!("{}{} {:?}", "error".bold().red(), ":".bold(), e);
        } else {
            eprintln!("{}{} {}", "error".bold().red(), ":".bold(), e);
        }
    }
}

async fn run_cli(matches: &ArgMatches) -> anyhow::Result<()> {
    if let Some(matches) = matches.subcommand_matches("generate") {
        generate::run_command(&matches)?;
    }

    if let Some(matches) = matches.subcommand_matches("install") {
        install::run_command(&matches).await?;
    }

    if let Some(matches) = matches.subcommand_matches("uninstall") {
        uninstall::run_command(&matches).await?;
    }

    if let Some(matches) = matches.subcommand_matches("inspect") {
        if let Some(matches) = matches.subcommand_matches("package") {
            inspect_package::run_command(&matches)?;
        }

        if let Some(matches) = matches.subcommand_matches("install") {
            inspect_install::run_command(&matches).await?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("registry") {
        if let Some(matches) = matches.subcommand_matches("add") {
            registry_add::run_command(matches)?;
        }

        if let Some(matches) = matches.subcommand_matches("list") {
            registry_list::run_command(&matches)?;
        }

        if let Some(matches) = matches.subcommand_matches("remove") {
            registry_remove::run_command(&matches)?;
        }
    }

    Ok(())
}
