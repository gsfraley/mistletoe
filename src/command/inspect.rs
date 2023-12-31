use crate::instance::MistPackageInstance;

use clap::ArgMatches;

pub fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let package = matches.get_one::<String>("package").unwrap();
    let mut instance = MistPackageInstance::load(&package, true)?;
    println!("{}", serde_yaml::to_string(&instance.info()?)?.trim());

    Ok(())
}
