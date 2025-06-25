/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use pyo3::prelude::*;

use ::anise::almanac::metaload::{MetaAlmanacError, MetaFile};

use std::env::consts::OS;
use std::fs;
use std::process::Command;

#[pyfunction]
pub(crate) fn exec_gui() -> Result<(), MetaAlmanacError> {
    if ["windows", "linux"].contains(&OS) {
        let crc32 = match OS {
            "linux" => Some(0x2046a9b7),
            "windows" => Some(0xac191672),
            _ => unreachable!(),
        };
        // Attempt to download from the public site.
        let mut gui = MetaFile {
            uri: format!("http://public-data.nyxspace.com/anise/v0.6/anise-gui-{OS}.exe"),
            crc32,
        };
        gui.process(true)?;
        make_executable(&gui.uri).expect("could not make ANISE GUI executable");
        // Now, execute this file.
        Command::new(gui.uri)
            .spawn()
            .expect("could not execute ANISE GUI");
        Ok(())
    } else {
        Err(MetaAlmanacError::FetchError {
            error: format!("{OS} not supported by ANISE GUI"),
            uri: format!("http://public-data.nyxspace.com/anise/v0.6/anise-gui-{OS}.exe"),
        })
    }
}

/// Sets the executable permission on the file if running on a Unix-like system.
/// Does nothing on other systems like Windows.
pub fn make_executable(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This function's signature is available on all platforms.
    // However, the code inside the block below is only included in the
    // binary when compiling for a Unix target.
    #[cfg(unix)]
    {
        // This platform-specific code requires the `PermissionsExt` trait.
        use std::os::unix::fs::PermissionsExt;

        // Get the existing permissions
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();

        // Add the execute permission using the octal mode
        let current_mode = permissions.mode();
        permissions.set_mode(current_mode | 0o111); // Add u+x, g+x, o+x

        // Apply the new permissions
        fs::set_permissions(path, permissions)?;
    }

    Ok(())
}
