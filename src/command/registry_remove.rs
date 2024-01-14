use crate::config::{ConfigLayout, RegistryLayout};
use crate::registry::process_registries;

use anyhow::anyhow;
use clap::ArgMatches;

pub fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let name = matches.get_one::<String>("name").unwrap();
    let mut config = ConfigLayout::from_env()?;

    let registry = config.spec.lookup_registry(name)
        .ok_or(anyhow!("could not find registry with the name \"{}\"", name))?;

    config.spec.registries = config.spec.registries.iter()
        .filter(|r| r.name != registry.name)
        .map(RegistryLayout::clone)
        .collect();

    config.write_to_env()?;
    process_registries(&config)?;

    Ok(())
}