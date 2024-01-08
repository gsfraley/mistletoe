use clap::ArgMatches;

use crate::config::{ConfigLayout, RegistryLayout, RemoteLayout, GitRemoteLayout};

pub fn run_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let name = matches.get_one::<String>("name").unwrap();
    let git = matches.get_one::<String>("git").unwrap();
    let mut config = ConfigLayout::from_env()?;

    let registry_layout = RegistryLayout {
        name: name.to_string(),
        default_remote: "default".to_string(),
        remotes: vec![RemoteLayout::Git {
            name: "default".to_string(),
            git: GitRemoteLayout {
                url: git.to_string(),
            },
        }],
    };

    config.spec.registries.push(registry_layout);
    config.write_to_env()?;

    Ok(())
}
