/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use self::error::{DataDecodingSnafu, DataSetLutSnafu};
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
use core::fmt;
use core::ops::Deref;
use der::{
    asn1::{OctetString, SequenceOf},
    Decode, Encode, Reader, Writer,
};
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

mod datatype;
mod error;

pub use datatype::DataSetType;
pub use error::DataSetError;

/// The kind of data that can be encoded in a dataset
pub trait DataSetT: Clone + Default + Encode + for<'a> Decode<'a> {
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
    pub data: Vec<T>,
}

impl<T: DataSetT, const ENTRIES: usize> DataSet<T, ENTRIES> {
    /// Try to load an Anise file from a pointer of bytes
    pub fn try_from_bytes<B: Deref<Target = [u8]>>(bytes: B) -> Result<Self, DataSetError> {
        match Self::from_der(&bytes) {
            Ok(ctx) => {
                trace!("[try_from_bytes] loaded context successfully");
                // Check the full integrity on load of the file.
                ctx.check_integrity().context(DataSetIntegritySnafu {
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
                    .context(DataDecodingSnafu {
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
        let bytes = self.build_data_seq().1;
        crc32fast::hash(bytes.as_bytes())
    }

    /// Sets the checksum of this data.
    /// NOTE: For this calculation, the data checksum field is set to u32::MAX;
    pub fn set_crc32(&mut self) {
        self.data_checksum = self.crc32();
    }

    pub fn check_integrity(&self) -> Result<(), IntegrityError> {
        // Ensure that the data is correctly decoded
        let computed_chksum = self.crc32();
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

    pub fn push(
        &mut self,
        item: T,
        id: Option<NaifId>,
        name: Option<&str>,
    ) -> Result<(), DataSetError> {
        let index = self.data.len() as u32;

        match id {
            Some(id) => {
                match name {
                    Some(name) => {
                        // Both an ID and a name
                        self.lut.append(id, name, index).context(DataSetLutSnafu {
                            action: "pushing data with ID and name",
                        })?;
                        // If the ID is the body of a system with a single object, also insert it for the system ID.
                        if [199, 299].contains(&id) {
                            self.lut.append(id / 100, name, index).context({
                                DataSetLutSnafu {
                                    action: "pushing data with ID and name",
                                }
                            })?;
                        }
                    }
                    None => {
                        // Only an ID and no name
                        self.lut.append_id(id, index).context(DataSetLutSnafu {
                            action: "pushing data with ID only",
                        })?;
                        // If the ID is the body of a system with a single object, also insert it for the system ID.
                        if [199, 299].contains(&id) {
                            self.lut.append_id(id / 100, index).context({
                                DataSetLutSnafu {
                                    action: "pushing data with ID and name",
                                }
                            })?;
                        }
                    }
                }
            }
            None => {
                if name.is_some() {
                    // Only a name
                    self.lut
                        .append_name(name.unwrap(), index)
                        .context(DataSetLutSnafu {
                            action: "pushing data with name only",
                        })?;
                } else {
                    return Err(DataSetError::DataSetLut {
                        action: "pushing data",
                        source: LutError::NoKeyProvided,
                    });
                }
            }
        }

        self.data.push(item);

        Ok(())
    }

    /// Get a copy of the data with that ID, if that ID is in the lookup table
    pub fn get_by_id(&self, id: NaifId) -> Result<T, DataSetError> {
        if let Some(index) = self.lut.by_id.get(&id) {
            // Found the ID
            self.data
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| LutError::InvalidIndex { index: *index })
                .context(DataSetLutSnafu {
                    action: "fetching by ID",
                })
        } else {
            Err(DataSetError::DataSetLut {
                action: "fetching by ID",
                source: LutError::UnknownId { id },
            })
        }
    }

    /// Mutates this dataset to change the value of the entry with that ID to the new provided value.
    /// This will return an error if the ID is not in the lookup table.
    /// Note that this function requires a new heap allocation to change the underlying dataset
    pub fn set_by_id(&mut self, id: NaifId, new_value: T) -> Result<(), DataSetError> {
        if let Some(index) = self.lut.by_id.get(&id) {
            *self
                .data
                .get_mut(*index as usize)
                .ok_or_else(|| LutError::InvalidIndex { index: *index })
                .context(DataSetLutSnafu {
                    action: "fetching by ID",
                })? = new_value;

            Ok(())
        } else {
            Err(DataSetError::DataSetLut {
                action: "setting by ID",
                source: LutError::UnknownId { id },
            })
        }
    }

    /// Mutates this dataset to remove the provided ID from the LUT and the dataset, removing also the lookup from its name if set.
    /// This will return an error if the ID is not in the lookup table.
    /// Note that this function requires a new heap allocation to change the underlying dataset
    pub fn rm_by_id(&mut self, id: NaifId) -> Result<(), DataSetError> {
        if let Some(index) = self.lut.by_id.remove(&id) {
            *self
                .data
                .get_mut(index as usize)
                .ok_or_else(|| LutError::InvalidIndex { index })
                .context(DataSetLutSnafu {
                    action: "fetching by ID",
                })? = T::default();

            // Search the names for that same entry.
            for (name, name_index) in &self.lut.by_name.clone() {
                if name_index == &index {
                    self.lut.rmname(name).context(DataSetLutSnafu {
                        action: "removing by ID",
                    })?;
                    break;
                }
            }

            Ok(())
        } else {
            Err(DataSetError::DataSetLut {
                action: "removing by ID",
                source: LutError::UnknownId { id },
            })
        }
    }

    /// Get a copy of the data with that name, if that name is in the lookup table
    pub fn get_by_name(&self, name: &str) -> Result<T, DataSetError> {
        if let Some(index) = self.lut.by_name.get(&name.try_into().unwrap()) {
            self.data
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| LutError::InvalidIndex { index: *index })
                .context(DataSetLutSnafu {
                    action: "fetching by name",
                })
        } else {
            Err(DataSetError::DataSetLut {
                action: "fetching by name",
                source: LutError::UnknownName {
                    name: name.try_into().unwrap(),
                },
            })
        }
    }

    /// Mutates this dataset to change the value of the entry with that name to the new provided value.
    /// This will return an error if the name is not in the lookup table.
    /// Note that this function requires a new heap allocation to change the underlying dataset
    pub fn set_by_name(&mut self, name: &str, new_value: T) -> Result<(), DataSetError> {
        if let Some(index) = self.lut.by_name.get(&name.try_into().unwrap()) {
            *self
                .data
                .get_mut(*index as usize)
                .ok_or_else(|| LutError::InvalidIndex { index: *index })
                .context(DataSetLutSnafu {
                    action: "fetching by ID",
                })? = new_value;

            Ok(())
        } else {
            Err(DataSetError::DataSetLut {
                action: "setting by name",
                source: LutError::UnknownName {
                    name: name.try_into().unwrap(),
                },
            })
        }
    }

    /// Mutates this dataset to remove the provided name from the LUT and the dataset, removing also the lookup from its ID if set.
    /// This will return an error if the name is not in the lookup table.
    /// Note that this function requires a new heap allocation to change the underlying dataset
    pub fn rm_by_name(&mut self, name: &str) -> Result<(), DataSetError> {
        if let Some(index) = self.lut.by_name.remove(&name.try_into().unwrap()) {
            *self
                .data
                .get_mut(index as usize)
                .ok_or_else(|| LutError::InvalidIndex { index })
                .context(DataSetLutSnafu {
                    action: "fetching by ID",
                })? = T::default();

            // Search the names for that same entry.
            for (id, id_index) in &self.lut.by_id.clone() {
                if id_index == &index {
                    self.lut.rmid(*id).context(DataSetLutSnafu {
                        action: "removing by name",
                    })?;
                    break;
                }
            }

            Ok(())
        } else {
            Err(DataSetError::DataSetLut {
                action: "removing by ID",
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
        self.lut.len()
    }

    /// Returns whether this dataset is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns this data as a data sequence, cloning all of the entries into this sequence.
    fn build_data_seq(&self) -> (Vec<u32>, OctetString) {
        let mut buf = Vec::with_capacity(ENTRIES * 2);
        let mut meta = Vec::with_capacity(ENTRIES * 2);
        meta.push(self.data.len() as u32);
        for data in &self.data {
            let mut this_buf = vec![];
            data.encode_to_vec(&mut this_buf).unwrap();
            meta.push(this_buf.len() as u32);
            buf.extend_from_slice(&this_buf);
        }
        let bytes = OctetString::new(buf).unwrap();
        (meta, bytes)
    }
}

impl<T: DataSetT, const ENTRIES: usize> Encode for DataSet<T, ENTRIES> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let (bytes_meta, bytes) = self.build_data_seq();
        self.metadata.encoded_len()?
            + self.lut.encoded_len()?
            + self.data_checksum.encoded_len()?
            + bytes_meta.encoded_len()?
            + bytes.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        let (bytes_meta, bytes) = self.build_data_seq();
        self.metadata.encode(encoder)?;
        self.lut.encode(encoder)?;
        self.data_checksum.encode(encoder)?;
        bytes_meta.encode(encoder)?;
        bytes.encode(encoder)
    }
}

impl<'a, T: DataSetT, const ENTRIES: usize> Decode<'a> for DataSet<T, ENTRIES> {
    fn decode<D: Reader<'a>>(decoder: &mut D) -> der::Result<Self> {
        let metadata = decoder.decode()?;
        let lut: LookUpTable<ENTRIES> = decoder.decode()?;
        let crc32_checksum = decoder.decode()?;
        // Metadata of the bytes to decode.
        // The first integer contains the number of usable items in the data.
        // The other integers are the encoded lengths of each of the data.
        let bytes_meta: SequenceOf<u32, ENTRIES> = decoder.decode()?;
        let der_octets: OctetString = decoder.decode()?;
        let bytes = der_octets.as_bytes();

        let mut data = vec![];

        let mut idx = 0;
        for meta_idx in 0..*bytes_meta.get(0).unwrap() as usize {
            let next_len = *bytes_meta.get(meta_idx + 1).unwrap() as usize;
            let this_data = T::from_der(&bytes[idx..idx + next_len]).unwrap();
            data.push(this_data);
            idx += next_len;
        }

        Ok(Self {
            metadata,
            lut,
            data_checksum: crc32_checksum,
            data,
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
    use std::mem::size_of;

    use crate::structure::{
        spacecraft::{DragData, Inertia, Mass, SRPData, SpacecraftData},
        SpacecraftDataSet,
    };

    use super::{DataSet, Decode, Encode};

    #[test]
    fn zero_repr() {
        // For this test, we want a data set with zero entries allowed in the LUT.
        let repr = DataSet::<SpacecraftData, 2>::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();
        assert_eq!(buf.len(), 63);

        let repr_dec = DataSet::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        assert_eq!(core::mem::size_of::<DataSet<SpacecraftData, 2>>(), 256);
        assert_eq!(core::mem::size_of::<DataSet<SpacecraftData, 128>>(), 8824);
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
        let end_idx = this_buf.len();
        // Build this entry data.
        let full_sc_entry = 0..end_idx;

        // Copy into the packed buffer
        for (i, byte) in this_buf.iter().enumerate() {
            packed_buf[i] = *byte;
        }
        // Check that we can decode what we have copied so far
        let full_sc_dec = SpacecraftData::from_der(&packed_buf[full_sc_entry]).unwrap();
        assert_eq!(full_sc_dec, full_sc);
        // Encode the other entry
        let mut this_buf = vec![];
        srp_sc.encode_to_vec(&mut this_buf).unwrap();
        // Copy into the packed buffer
        for (i, byte) in this_buf.iter().enumerate() {
            packed_buf[i + end_idx] = *byte;
        }
        let srp_sc_entry = end_idx..end_idx + this_buf.len();
        // Check that we can decode the next entry
        let srp_sc_dec = SpacecraftData::from_der(&packed_buf[srp_sc_entry]).unwrap();
        assert_eq!(srp_sc_dec, srp_sc);
        // Build the dataset
        let mut dataset = DataSet::default();

        // Build the lookup table
        dataset
            .push(srp_sc.clone(), Some(-20), Some("SRP spacecraft"))
            .unwrap();
        dataset
            .push(full_sc.clone(), Some(-50), Some("Full spacecraft"))
            .unwrap();

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
        assert!(repr_dec.get_by_id(0).is_err());

        // Check that we can modify it.
        let orig_dataset = dataset.clone();

        // Grab a copy of the original data
        let mut sc = dataset.get_by_name("SRP spacecraft").unwrap();
        sc.srp_data.as_mut().unwrap().coeff_reflectivity = 1.1;
        dataset.set_by_name("SRP spacecraft", sc.clone()).unwrap();
        // Ensure that we've modified only that entry
        assert_eq!(
            dataset.get_by_name("Full spacecraft").unwrap(),
            orig_dataset.get_by_name("Full spacecraft").unwrap(),
            "immutable value was modified"
        );
        // Ensure that we've modified the entry we wanted to modify
        assert_eq!(
            dataset
                .get_by_name("SRP spacecraft")
                .unwrap()
                .srp_data
                .unwrap()
                .coeff_reflectivity,
            1.1,
            "value was not modified"
        );
        assert!(dataset.set_by_name("Unavailable SC", sc.clone()).is_err());

        // Test renaming by name
        dataset
            .lut
            .rename("SRP spacecraft", "Renamed SRP spacecraft")
            .unwrap();
        // Calling this a second time will lead to an error
        assert!(dataset
            .lut
            .rename("SRP spacecraft", "Renamed SRP spacecraft")
            .is_err());
        // Calling the original will lead to an error
        assert!(dataset.get_by_name("SRP spacecraft").is_err());
        // Check that we can fetch that data as we modified it.
        assert_eq!(
            dataset
                .get_by_name("Renamed SRP spacecraft")
                .unwrap()
                .srp_data
                .unwrap()
                .coeff_reflectivity,
            1.1,
            "value not reachable after rename"
        );
        // Finally remove that ID all together and make sure it is not reachable.
        assert!(dataset.lut.rmname("Renamed SRP spacecraft").is_ok());
        // Second call fails
        assert!(dataset.lut.rmname("Renamed SRP spacecraft").is_err());
        // Fetch fails
        assert!(dataset.get_by_name("Renamed SRP spacecraft").is_err());
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

        dbg!(size_of::<SpacecraftDataSet>());

        let mut dataset = DataSet::<SpacecraftData, 16>::default();
        dataset
            .push(srp_sc.clone(), Some(-20), Some("SRP spacecraft"))
            .unwrap();

        dataset
            .push(full_sc.clone(), Some(-50), Some("Full spacecraft"))
            .unwrap();

        // Pushing without name as ID -51
        dataset.push(full_sc.clone(), Some(-51), None).unwrap();

        // Pushing without ID
        dataset
            .push(srp_sc.clone(), None, Some("ID less SRP spacecraft"))
            .unwrap();

        // Make sure to set the CRC32.
        dataset.set_crc32();
        // And encode it.

        let mut ebuf = vec![];
        dataset.encode_to_vec(&mut ebuf).unwrap();

        // assert_eq!(ebuf.len(), 506);

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

        // Check that we can set by ID
        let mut repr = dataset.get_by_id(-50).unwrap();
        repr.mass_kg.as_mut().unwrap().dry_mass_kg = 100.5;
        dataset.set_by_id(-50, repr.clone()).unwrap();
        assert_eq!(
            dataset.get_by_id(-50).unwrap().mass_kg.unwrap().dry_mass_kg,
            100.5,
            "value was not modified"
        );
        assert!(dataset.set_by_id(111, repr.clone()).is_err());
        // Test renaming by ID
        dataset.lut.reid(-50, -52).unwrap();
        // Calling this a second time will lead to an error
        assert!(dataset.lut.reid(-50, -52).is_err());
        // Calling the original will lead to an error
        assert!(dataset.get_by_id(-50).is_err());
        // Check that we can fetch that data as we modified it.
        assert_eq!(
            dataset.get_by_id(-52).unwrap().mass_kg.unwrap().dry_mass_kg,
            100.5,
            "value not reachable after reid"
        );
        // Finally remove that ID all together and make sure it is not reachable.
        assert!(dataset.lut.rmid(-52).is_ok());
        // Second call fails
        assert!(dataset.lut.rmid(-52).is_err());
        // Fetch fails
        assert!(dataset.get_by_id(-52).is_err());

        // Remove by ID
        assert!(dataset.rm_by_id(-20).is_ok(), "could not remove by id");
        // Check that the associated name is no reachable
        assert!(
            dataset.get_by_name("SRP spacecraft").is_err(),
            "still reachable by name"
        );

        // Remove by name
        assert!(
            dataset.rm_by_name("Full spacecraft").is_ok(),
            "could not remove by name"
        );
        // Check that the associated name is no reachable
        assert!(dataset.get_by_id(-52).is_err(), "still reachable by id");
    }
}
