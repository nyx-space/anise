/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

mod metaalmanac;
mod metafile;

pub use metaalmanac::MetaAlmanac;
pub use metafile::MetaFile;

use super::Almanac;

use crate::prelude::InputOutputError;
use reqwest::StatusCode;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum MetaAlmanacError {
    #[snafu(display("could not create the cache folder for ANISE, please use a relative path"))]
    AppDirError,
    #[snafu(display("could not find a file path in {path}"))]
    MissingFilePath { path: String },
    #[snafu(display("IO error {source} when {what} with {path}"))]
    MetaIO {
        path: String,
        what: &'static str,
        source: InputOutputError,
    },
    #[snafu(display("fetching {uri} returned {status}"))]
    FetchError { status: StatusCode, uri: String },
    #[snafu(display("connection {uri} returned {error}"))]
    CnxError { uri: String, error: String },
    #[snafu(display("error parsing `{path}` as Dhall config: {err}"))]
    ParseDhall { path: String, err: String },
    #[snafu(display("error exporting as Dhall config (please file a bug): {err}"))]
    ExportDhall { err: String },
}

#[cfg(test)]
mod meta_test {
    use super::MetaAlmanac;
    use std::path::Path;
    use std::{env, str::FromStr};

    #[test]
    fn test_meta_almanac() {
        let _ = pretty_env_logger::try_init();
        let mut meta = MetaAlmanac::default();
        println!("{meta:?}");

        let almanac = meta._process().unwrap();
        // Shows everything in this Almanac
        almanac.describe(None, None, None, None);

        // Process again to confirm that the CRC check works
        assert!(meta._process().is_ok());
    }

    #[test]
    fn test_from_dhall() {
        let default = MetaAlmanac::default();

        println!("{}", default.dump().unwrap());

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/latest.dhall");
        let dhall = MetaAlmanac::new(path.to_str().unwrap().to_string()).unwrap();

        assert_eq!(dhall, default);

        // Try FromStr

        let from_str = MetaAlmanac::from_str(
            r#"
 { files =
   [ { crc32 = Some 1921414410
     , uri = "http://public-data.nyxspace.com/anise/de440s.bsp"
     }
   , { crc32 = Some 1216081528
     , uri = "http://public-data.nyxspace.com/anise/pck08.pca"
     }
   , { crc32 = Some 1817759242
     , uri = "http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc"
     }
   , { crc32 = None Natural
     , uri =
         "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
     }
   ]
 }
             "#,
        )
        .unwrap();

        assert_eq!(from_str, default);
    }
}
