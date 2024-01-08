use crate::config::ConfigLayout;

use clap::ArgMatches;

pub fn run_command(_: &ArgMatches) -> anyhow::Result<()> {
    println!("{}", serde_yaml::to_string(&ConfigLayout::from_env()?.spec.registries)?);
    Ok(())
}
