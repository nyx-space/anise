use crate::{
    almanac::Almanac,
    errors::{AlmanacError, AlmanacResult, TLDataSetSnafu},
    structure::PlanetaryDataSet,
};
use bytes::Bytes;
use rust_embed::Embed;
use snafu::ResultExt;

#[derive(Embed)]
#[cfg_attr(not(docsrs), folder = "$CARGO_MANIFEST_DIR/../data/")]
#[cfg_attr(not(docsrs), include = "de440s.bsp")]
#[cfg_attr(not(docsrs), include = "pck11.pca")]
#[cfg_attr(docsrs, folder = "$OUT_DIR")]
struct AstroData;

impl Almanac {
    /// Provides planetary ephemerides from 2024-01-01 until 2035-01-01. Also provides planetary constants data (from the PCK11 kernel).
    ///
    /// Until <https://github.com/nyx-space/anise/issues/269>, this will provide 100 years of data
    pub fn until_2035() -> AlmanacResult<Self> {
        // Regularly refer to https://github.com/nyx-space/anise/blob/master/data/ci_config.dhall for the latest CRC, although it should not change between minor versions!
        let pck11 = AstroData::get("pck11.pca").ok_or(AlmanacError::GenericError {
            err: "could not find pck11.pca in embedded files".to_string(),
        })?;
        let almanac = Almanac {
            planetary_data: PlanetaryDataSet::try_from_bytes(pck11.data.as_ref()).context(
                TLDataSetSnafu {
                    action: "loading PCK11 from embedded file",
                },
            )?,
            ..Default::default()
        };

        let pl_ephem = AstroData::get("de440s.bsp").ok_or(AlmanacError::GenericError {
            err: "could not find de440s.bsp in embedded files".to_string(),
        })?;

        almanac.load_from_bytes(Bytes::copy_from_slice(pl_ephem.data.as_ref()))
    }
}

#[cfg(test)]
mod ut_embed {
    use super::{Almanac, AstroData};

    #[test]
    fn test_embedded_load() {
        let almanac = Almanac::until_2035().unwrap();
        assert_eq!(almanac.num_loaded_spk(), 1);
        assert_eq!(almanac.num_loaded_bpc(), 0);
        assert_ne!(almanac.planetary_data.crc32(), 0);
    }

    #[test]
    fn test_limited_set() {
        // Check only PCK11 is present
        assert!(AstroData::get("pck11.pca").is_some());
        assert!(AstroData::get("pck08.pca").is_none());
        // Check only one planetary ephem is present
        assert!(AstroData::get("de440s.bsp").is_some());
        assert!(AstroData::get("de440.bsp").is_none());
    }
}
