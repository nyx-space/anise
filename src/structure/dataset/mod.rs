/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use self::error::DataDecodingSnafu;
use super::{
    lookuptable::{LookUpTable, LutError},
    metadata::Metadata,
    semver::Semver,
    ANISE_VERSION,
};
use crate::{
    errors::{DecodingError, IntegrityError},
    structure::dataset::error::DataSetIntegritySnafu,
    NaifId,
};
use bytes::Bytes;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use der::{asn1::OctetStringRef, Decode, Encode, Reader, Writer};
use log::{error, trace};
use snafu::prelude::*;

macro_rules! io_imports {
    () => {
        use std::fs::File;
        use std::io::{Error as IOError, ErrorKind as IOErrorKind, Write};
        use std::path::Path;
        use std::path::PathBuf;
    };
}

io_imports!();

mod builder;
mod datatype;
mod error;

pub use builder::DataSetBuilder;
pub use datatype::DataSetType;
pub use error::DataSetError;

/// The kind of data that can be encoded in a dataset
pub trait DataSetT: Encode + for<'a> Decode<'a> {
    const NAME: &'static str;
}

/// A DataSet is the core structure shared by all ANISE binary data.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DataSet<T: DataSetT, const ENTRIES: usize> {
    pub metadata: Metadata,
    /// All datasets have LookUpTable (LUT) that stores the mapping between a key and its index in the ephemeris list.
    pub lut: LookUpTable<ENTRIES>,
    pub data_checksum: u32,
    /// The actual data from the dataset
    pub bytes: Bytes,
    _daf_type: PhantomData<T>,
}

impl<T: DataSetT, const ENTRIES: usize> DataSet<T, ENTRIES> {
    /// Try to load an Anise file from a pointer of bytes
    pub fn try_from_bytes<B: Deref<Target = [u8]>>(bytes: B) -> Result<Self, DataSetError> {
        match Self::from_der(&bytes) {
            Ok(ctx) => {
                trace!("[try_from_bytes] loaded context successfully");
                // Check the full integrity on load of the file.
                ctx.check_integrity()
                    .with_context(|_| DataSetIntegritySnafu {
                        action: "loading data set from bytes",
                    })?;
                Ok(ctx)
            }
            Err(_) => {
                // If we can't load the file, let's try to load the version only to be helpful
                let semver_bytes = bytes
                    .get(0..5)
                    .ok_or(DecodingError::InaccessibleBytes {
                        start: 0,
                        end: 5,
                        size: bytes.len(),
                    })
                    .with_context(|_| DataDecodingSnafu {
                        action: "checking data set version",
                    })?;
                match Semver::from_der(semver_bytes) {
                    Ok(file_version) => {
                        if file_version == ANISE_VERSION {
                            Err(DataSetError::DataDecoding {
                                action: "loading from bytes",
                                source: DecodingError::Obscure { kind: T::NAME },
                            })
                        } else {
                            Err(DataSetError::DataDecoding {
                                action: "checking data set version",
                                source: DecodingError::AniseVersion {
                                    got: file_version,
                                    exp: ANISE_VERSION,
                                },
                            })
                        }
                    }
                    Err(err) => {
                        error!("context bytes not in ANISE format");
                        Err(DataSetError::DataDecoding {
                            action: "loading SemVer",
                            source: DecodingError::DecodingDer { err },
                        })
                    }
                }
            }
        }
    }

    /// Forces to load an Anise file from a pointer of bytes.
    /// **Panics** if the bytes cannot be interpreted as an Anise file.
    pub fn from_bytes<B: Deref<Target = [u8]>>(buf: B) -> Self {
        Self::try_from_bytes(buf).unwrap()
    }

    /// Compute the CRC32 of the underlying bytes
    pub fn crc32(&self) -> u32 {
        crc32fast::hash(&self.bytes)
    }

    /// Sets the checksum of this data.
    pub fn set_crc32(&mut self) {
        self.data_checksum = self.crc32();
    }

    pub fn check_integrity(&self) -> Result<(), IntegrityError> {
        // Ensure that the data is correctly decoded
        let computed_chksum = crc32fast::hash(&self.bytes);
        if computed_chksum == self.data_checksum {
            Ok(())
        } else {
            error!(
                "[integrity] expected hash {} but computed {}",
                self.data_checksum, computed_chksum
            );
            Err(IntegrityError::ChecksumInvalid {
                expected: self.data_checksum,
                computed: computed_chksum,
            })
        }
    }

    /// Scrubs the data by computing the CRC32 of the bytes and making sure that it still matches the previously known hash
    pub fn scrub(&self) -> Result<(), IntegrityError> {
        if self.crc32() == self.data_checksum {
            Ok(())
        } else {
            // Compiler will optimize the double computation away
            Err(IntegrityError::ChecksumInvalid {
                expected: self.data_checksum,
                computed: self.crc32(),
            })
        }
    }

    pub fn get_by_id(&self, id: NaifId) -> Result<T, DataSetError> {
        if let Some(entry) = self.lut.by_id.get(&id) {
            // Found the ID
            let bytes = self
                .bytes
                .get(entry.as_range())
                .ok_or_else(|| entry.decoding_error())
                .with_context(|_| DataDecodingSnafu {
                    action: "fetching by ID",
                })?;
            T::from_der(bytes)
                .map_err(|err| DecodingError::DecodingDer { err })
                .with_context(|_| DataDecodingSnafu {
                    action: "fetching by ID",
                })
        } else {
            Err(DataSetError::DataSetLut {
                action: "fetching by ID",
                source: LutError::UnknownId { id },
            })
        }
    }

    pub fn get_by_name(&self, name: &str) -> Result<T, DataSetError> {
        if let Some(entry) = self.lut.by_name.get(&name.try_into().unwrap()) {
            // Found the name
            let bytes = self
                .bytes
                .get(entry.as_range())
                .ok_or_else(|| entry.decoding_error())
                .with_context(|_| DataDecodingSnafu {
                    action: "fetching by name",
                })?;
            T::from_der(bytes)
                .map_err(|err| DecodingError::DecodingDer { err })
                .with_context(|_| DataDecodingSnafu {
                    action: "fetching by name",
                })
        } else {
            Err(DataSetError::DataSetLut {
                action: "fetching by ID",
                source: LutError::UnknownName {
                    name: name.try_into().unwrap(),
                },
            })
        }
    }

    /// Saves this dataset to the provided file
    /// If overwrite is set to false, and the filename already exists, this function will return an error.

    pub fn save_as(&self, filename: &PathBuf, overwrite: bool) -> Result<(), DataSetError> {
        use log::{info, warn};

        if Path::new(&filename).exists() {
            if !overwrite {
                return Err(DataSetError::IO {
                    source: IOError::new(
                        IOErrorKind::AlreadyExists,
                        "file exists and overwrite flag set to false",
                    ),
                    action: "creating data set file",
                });
            } else {
                warn!("[save_as] overwriting {}", filename.display());
            }
        }

        let mut buf = vec![];

        match File::create(filename) {
            Ok(mut file) => {
                if let Err(err) = self.encode_to_vec(&mut buf) {
                    return Err(DataSetError::DataDecoding {
                        action: "encoding data set",
                        source: DecodingError::DecodingDer { err },
                    });
                }
                if let Err(source) = file.write_all(&buf) {
                    Err(DataSetError::IO {
                        source,
                        action: "writing data set to file",
                    })
                } else {
                    info!("[OK] dataset saved to {}", filename.display());
                    Ok(())
                }
            }
            Err(source) => Err(DataSetError::IO {
                source,
                action: "creating data set file",
            }),
        }
    }

    /// Returns the length of the LONGEST of the two look up tables
    pub fn len(&self) -> usize {
        if self.lut.by_id.len() > self.lut.by_name.len() {
            self.lut.by_id.len()
        } else {
            self.lut.by_name.len()
        }
    }

    /// Returns whether this dataset is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: DataSetT, const ENTRIES: usize> Encode for DataSet<T, ENTRIES> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let as_byte_ref = OctetStringRef::new(&self.bytes)?;
        self.metadata.encoded_len()?
            + self.lut.encoded_len()?
            + self.data_checksum.encoded_len()?
            + as_byte_ref.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        let as_byte_ref = OctetStringRef::new(&self.bytes)?;
        self.metadata.encode(encoder)?;
        self.lut.encode(encoder)?;
        self.data_checksum.encode(encoder)?;
        as_byte_ref.encode(encoder)
    }
}

impl<'a, T: DataSetT, const ENTRIES: usize> Decode<'a> for DataSet<T, ENTRIES> {
    fn decode<D: Reader<'a>>(decoder: &mut D) -> der::Result<Self> {
        let metadata = decoder.decode()?;
        let lut = decoder.decode()?;
        let crc32_checksum = decoder.decode()?;
        let bytes: OctetStringRef = decoder.decode()?;
        Ok(Self {
            metadata,
            lut,
            data_checksum: crc32_checksum,
            bytes: Bytes::copy_from_slice(bytes.as_bytes()),
            _daf_type: PhantomData::<T>,
        })
    }
}

impl<T: DataSetT, const ENTRIES: usize> fmt::Display for DataSet<T, ENTRIES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} with {} ID mappings and {} name mappings",
            self.metadata.dataset_type,
            self.lut.by_id.len(),
            self.lut.by_name.len()
        )
    }
}

#[cfg(test)]
mod dataset_ut {
    use crate::structure::{
        dataset::DataSetBuilder,
        lookuptable::Entry,
        spacecraft::{DragData, Inertia, Mass, SRPData, SpacecraftData},
        SpacecraftDataSet,
    };
    use bytes::Bytes;

    use super::{DataSet, Decode, Encode, LookUpTable};

    #[test]
    fn zero_repr() {
        // For this test, we want a data set with zero entries allowed in the LUT.
        let repr = DataSet::<SpacecraftData, 2>::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();
        assert_eq!(buf.len(), 58);

        let repr_dec = DataSet::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        assert_eq!(core::mem::size_of::<DataSet<SpacecraftData, 2>>(), 288);
        assert_eq!(core::mem::size_of::<DataSet<SpacecraftData, 128>>(), 10368);
    }

    #[test]
    fn spacecraft_constants_lookup() {
        // Build some data first.
        let full_sc = SpacecraftData {
            name: "full spacecraft".try_into().unwrap(),
            srp_data: Some(SRPData {
                area_m2: 2.0,
                coeff_reflectivity: 1.8,
            }),
            inertia: Some(Inertia {
                orientation_id: -20,
                i_11_kgm2: 120.0,
                i_22_kgm2: 180.0,
                i_33_kgm2: 220.0,
                i_12_kgm2: 20.0,
                i_13_kgm2: -15.0,
                i_23_kgm2: 30.0,
            }),
            mass_kg: Some(Mass::from_dry_and_fuel_masses(150.0, 50.6)),
            drag_data: Some(DragData::default()),
        };
        let srp_sc = SpacecraftData {
            name: "SRP only spacecraft".try_into().unwrap(),
            srp_data: Some(SRPData::default()),
            ..Default::default()
        };

        // Pack the data into the vector (encoding will likely always require allocation).
        let mut packed_buf = [0; 1000];

        let mut this_buf = vec![];
        full_sc.encode_to_vec(&mut this_buf).unwrap();
        let end_idx = this_buf.len() as u32;
        // Build this entry data.
        let full_sc_entry = Entry {
            start_idx: 0,
            end_idx,
        };
        // Copy into the packed buffer
        for (i, byte) in this_buf.iter().enumerate() {
            packed_buf[i] = *byte;
        }
        // Check that we can decode what we have copied so far
        let full_sc_dec = SpacecraftData::from_der(&packed_buf[full_sc_entry.as_range()]).unwrap();
        assert_eq!(full_sc_dec, full_sc);
        // Encode the other entry
        let mut this_buf = vec![];
        srp_sc.encode_to_vec(&mut this_buf).unwrap();
        // Copy into the packed buffer
        for (i, byte) in this_buf.iter().enumerate() {
            packed_buf[i + end_idx as usize] = *byte;
        }
        let srp_sc_entry = Entry {
            start_idx: end_idx,
            end_idx: end_idx + this_buf.len() as u32,
        };
        // Check that we can decode the next entry
        let srp_sc_dec = SpacecraftData::from_der(&packed_buf[srp_sc_entry.as_range()]).unwrap();
        assert_eq!(srp_sc_dec, srp_sc);
        // Build the lookup table
        let mut lut = LookUpTable::default();
        lut.append(-20, "SRP spacecraft", srp_sc_entry).unwrap();
        lut.append(-50, "Full spacecraft", full_sc_entry).unwrap();
        // Build the dataset
        let mut dataset = DataSet {
            lut,
            bytes: Bytes::copy_from_slice(&packed_buf),
            ..Default::default()
        };
        dataset.set_crc32();
        // And encode it.

        let mut buf = vec![];
        dataset.encode_to_vec(&mut buf).unwrap();

        let repr_dec = DataSet::<SpacecraftData, 4>::from_der(&buf).unwrap();

        assert_eq!(dataset, repr_dec);

        assert!(repr_dec.check_integrity().is_ok());

        // Now that the data is valid, let's fetch the data back

        let full_sc_repr = repr_dec.get_by_id(-50).unwrap();
        assert_eq!(full_sc_repr, full_sc);

        let srp_repr = repr_dec.get_by_id(-20).unwrap();
        assert_eq!(srp_repr, srp_sc);

        // And check that we get an error if the data is wrong.
        assert!(repr_dec.get_by_id(0).is_err())
    }

    #[test]
    fn spacecraft_constants_lookup_builder() {
        // Build some data first.
        let full_sc = SpacecraftData {
            name: "full spacecraft".try_into().unwrap(),
            srp_data: Some(SRPData {
                area_m2: 2.0,
                coeff_reflectivity: 1.8,
            }),
            inertia: Some(Inertia {
                orientation_id: -20,
                i_11_kgm2: 120.0,
                i_22_kgm2: 180.0,
                i_33_kgm2: 220.0,
                i_12_kgm2: 20.0,
                i_13_kgm2: -15.0,
                i_23_kgm2: 30.0,
            }),
            mass_kg: Some(Mass::from_dry_and_fuel_masses(150.0, 50.6)),
            drag_data: Some(DragData::default()),
        };
        let srp_sc = SpacecraftData {
            name: "SRP only spacecraft".try_into().unwrap(),
            srp_data: Some(SRPData::default()),
            ..Default::default()
        };

        // Initialize the overall buffer for building the data
        let mut buf = vec![];
        let mut builder = DataSetBuilder::default();
        builder
            .push_into(&mut buf, &srp_sc, Some(-20), Some("SRP spacecraft"))
            .unwrap();

        builder
            .push_into(&mut buf, &full_sc, Some(-50), Some("Full spacecraft"))
            .unwrap();

        // Pushing without name as ID -51
        builder
            .push_into(&mut buf, &full_sc, Some(-51), None)
            .unwrap();

        // Pushing without ID
        builder
            .push_into(&mut buf, &srp_sc, None, Some("ID less SRP spacecraft"))
            .unwrap();

        let dataset = builder.finalize(buf).unwrap();

        // And encode it.

        let mut ebuf = vec![];
        dataset.encode_to_vec(&mut ebuf).unwrap();

        assert_eq!(ebuf.len(), 530);

        let repr_dec = SpacecraftDataSet::from_bytes(ebuf);

        assert_eq!(dataset, repr_dec);

        assert!(repr_dec.check_integrity().is_ok());

        // Now that the data is valid, let's fetch the data back

        let full_sc_repr = repr_dec.get_by_id(-50).unwrap();
        assert_eq!(full_sc_repr, full_sc);

        let srp_repr = repr_dec.get_by_id(-20).unwrap();
        assert_eq!(srp_repr, srp_sc);

        // And check that we get an error if the data is wrong.
        assert!(repr_dec.get_by_id(0).is_err());
    }
}
