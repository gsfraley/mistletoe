use crate::installation::InstallRef;
use crate::outputs::*;

use anyhow::anyhow;
use clap::ArgMatches;
use kube::core::DynamicObject;

pub async fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let name = matches.get_one::<String>("name").unwrap();

    let output_mode = match matches.get_one::<String>("output").map(|o| o.as_str()) {
        None | Some("list") => OutputMode::List,
        Some("yaml") => OutputMode::Yaml,
        Some(o) => Err(anyhow!("Unexpected output type: {}", o))?,
    };

    let install_ref = InstallRef { name: name.to_string(), version: None };
    let resources = install_ref.get_resources().await?;

    output_resources(resources, output_mode, name)?;

    Ok(())
}

enum OutputMode {
    Yaml,
    List,
}

fn output_resources(resources: Vec<DynamicObject>, mode: OutputMode, name: &str) -> anyhow::Result<()> {
    if resources.is_empty() {
        return Err(anyhow!("no resources found for installation name \"{}\"", name));
    }

    match mode {
        OutputMode::Yaml => Ok(println!("{}", resources.mc_output_yaml()?)),
        OutputMode::List => Ok(println!("{}", resources.mc_output_list()?)),
    }
}
