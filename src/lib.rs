pub mod husk;

use std::path::PathBuf;

use mistletoe_api::v0_1::{MistHuskResult, serialize_result};

pub enum OutputMode {
    Raw,
    Yaml,
    Dir(PathBuf),
}

pub fn output_result(result: MistHuskResult, mode: OutputMode) -> anyhow::Result<()> {
    match mode {
        OutputMode::Raw => println!("{}", serialize_result(result)?),
        OutputMode::Yaml => match result {
            Ok(output) => output.as_files().values()
                .for_each(|content| println!("{}", content)),
            Err(e) => { return Err(e) }
        },
        OutputMode::Dir(_path) => todo!("Implement outputting a directory structure"),
    };

    Ok(())
}
