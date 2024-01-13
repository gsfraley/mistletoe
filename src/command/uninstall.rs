use crate::installation::InstallRef;
use crate::outputs::*;

use anyhow::anyhow;
use clap::ArgMatches;
use kube::core::DynamicObject;

pub async fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let name = matches.get_one::<String>("name").unwrap();

    let install_ref = InstallRef { name: name.to_string(), version: None };
    let resources = install_ref.delete_resources().await?;
    output_resources(resources, name)?;

    Ok(())
}

fn output_resources(resources: Vec<DynamicObject>, name: &str) -> anyhow::Result<()> {
    if resources.is_empty() {
        return Err(anyhow!("no resources found for installation name \"{}\"", name));
    }

    Ok(println!("{}", resources.mc_output_list()?
        .lines().into_iter()
        .map(|line| format!("deleted {}", line))
        .collect::<Vec<String>>()
        .join("\n")))
}
