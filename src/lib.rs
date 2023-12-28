pub mod husk;
pub mod registry;

use std::fs;
use std::path::PathBuf;

use mistletoe_api::v0_1::{MistHuskResult, serialize_result};

pub enum OutputMode {
    Raw,
    Yaml,
    Dir(PathBuf),
}

pub fn output_result(result: MistHuskResult, mode: OutputMode) -> anyhow::Result<()> {
    if let Ok(output) = &result {
        if let Some(message) = output.get_message() {
            println!("{}", message);
        }
    }

    match mode {
        OutputMode::Raw => Ok(println!("{}", serialize_result(result)?)),

        OutputMode::Yaml => match result {
            Ok(output) => {
                output.get_files().values()
                    .for_each(|content| println!("{}", content.trim()));

                Ok(())
            },
            Err(e) => Err(e),
        },

        OutputMode::Dir(path) => match result {
            Ok(output) => {
                for (filename, content) in output.get_files() {
                    let out_path = path.join(PathBuf::from(filename));
                    fs::write(out_path, content)?;
                }

                Ok(())
            },
            Err(e) => Err(e),
        },
    }
}
