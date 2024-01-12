use crate::installation::InstallRef;

use clap::ArgMatches;

pub async fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let name = matches.get_one::<String>("name").unwrap();

    let install_ref = InstallRef { name: name.to_string(), version: None };
    install_ref.delete_resources().await?;

    Ok(())
}
