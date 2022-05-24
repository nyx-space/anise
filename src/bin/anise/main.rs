use anise::cli::args::{Actions, Args};
use anise::cli::CliErrors;
use anise::file_mmap;
use anise::naif::daf::DAF;
use anise::naif::spk::SPK;
use anise::prelude::*;
use clap::Parser;

fn main() -> Result<(), CliErrors> {
    let cli = Args::parse();
    match cli.action {
        Actions::Convert { file } => {
            let file_clone = file.clone();
            // Memory map the file
            let bytes = file_mmap!(file).unwrap();

            let daf_file = DAF::parse(&bytes)?;
            // Parse as SPK
            let spk: SPK = (&daf_file).try_into()?;
            // Convert to ANISE
            let spk_filename = file_clone.to_str().unwrap();
            let anise_filename = spk_filename.replace(".bsp", ".anise");
            spk.to_anise(spk_filename, &anise_filename);
            Ok(())
        }
        Actions::IntegrityCheck { file: _ } => {
            todo!()
        }
        Actions::Merge { files } => {
            todo!("merge {files:?}")
        }
    }
}
