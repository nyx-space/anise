extern crate env_logger;
use std::fs::rename;

use anise::cli::args::{Actions, Args};
use anise::cli::CliErrors;
use anise::file_mmap;
use anise::naif::daf::DAF;
use anise::naif::spk::SPK;
use anise::prelude::*;
use clap::Parser;

fn main() -> Result<(), CliErrors> {
    env_logger::init();

    let cli = Args::parse();
    match cli.action {
        Actions::Convert { file } => {
            let file_clone = file.clone();
            // Memory map the file
            match file_mmap!(file) {
                Ok(bytes) => {
                    let daf_file = DAF::parse(&bytes)?;
                    // Parse as SPK
                    let spk: SPK = (&daf_file).try_into()?;
                    // Convert to ANISE
                    let spk_filename = file_clone.to_str().unwrap();
                    let anise_filename = spk_filename.replace(".bsp", ".anise");
                    spk.to_anise(spk_filename, &anise_filename)?;
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Actions::Check { file } => {
            let path_str = file.clone();
            match file_mmap!(file) {
                Ok(bytes) => {
                    let context = AniseContext::try_from_bytes(&bytes)?;
                    context.check_integrity()?;
                    println!("[OK] {:?}", path_str);
                    Ok(())
                }
                Err(e) => return Err(e.into()),
            }
        }
        Actions::Inspect { file: _ } => {
            todo!()
        }
        Actions::Merge { files } => {
            if files.len() < 2 {
                Err(CliErrors::ArgumentError(
                    "Need at least two files to merge together".to_string(),
                ))
            } else {
                // Open the last file in the list
                let destination = files.last().unwrap().clone();
                // This is the temporary file.
                let dest_str = files.last().unwrap().to_str().as_ref().unwrap().to_string();
                let tmp_dest_str = dest_str.clone() + ".tmp";
                match file_mmap!(destination) {
                    Ok(bytes) => {
                        let mut context = AniseContext::try_from_bytes(&bytes)?;
                        // We can't borrow some bytes and let them drop out of scope, so we'll copy them into a vector.
                        let mut all_bytes = Vec::with_capacity(files.len() - 1);
                        for this_file in files.iter().take(files.len() - 1) {
                            // Try load this file
                            match file_mmap!(this_file) {
                                Ok(these_bytes) => {
                                    all_bytes.push(these_bytes);
                                }
                                Err(e) => return Err(e.into()),
                            }
                        }
                        // And now add them to the previous context
                        for (num, bytes) in all_bytes.iter().enumerate() {
                            let other = AniseContext::try_from_bytes(bytes)?;
                            let (num_ephem_added, num_orientation_added) =
                                context.merge_mut(other.clone())?;
                            println!("Added {num_ephem_added} ephemeris and {num_orientation_added} orientations from {:?}", files[num]);
                        }
                        // And finally save.
                        if let Err(e) = context.save_as(&tmp_dest_str, false) {
                            return Err(e.into());
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
                // Now that we have written the data to the temp file
                // and that the mmap is out of scope, we can move the tmp file into the old file
                if let Err(e) = rename(tmp_dest_str, dest_str) {
                    Err(CliErrors::AniseError(AniseError::IOError(e.kind())))
                } else {
                    Ok(())
                }
            }
        }
    }
}
