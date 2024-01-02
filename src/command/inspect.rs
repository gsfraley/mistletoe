use crate::loader;

use clap::ArgMatches;

pub fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let package = matches.get_one::<String>("package").unwrap();
    let package_ref = loader::PackageRef::from_str(package)?;
    let mut instance = loader::load(&package_ref)?;
    println!("{}", serde_yaml::to_string(&instance.info()?)?.trim());

    Ok(())
}
