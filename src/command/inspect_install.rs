use clap::ArgMatches;

use crate::installation::InstallRef;

pub async fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let name = matches.get_one::<String>("name").unwrap();

    let install_ref = InstallRef { name: name.to_string(), version: None };
    let resources = install_ref.get_resources().await?;

    println!("{}", resources.iter().map(serde_yaml::to_string)
        .collect::<Result<Vec<String>, _>>()?
        .join("\n---\n").trim());

    Ok(())
}
