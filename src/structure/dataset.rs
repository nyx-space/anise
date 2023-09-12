/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use super::{
    lookuptable::{Entry, LookUpTable},
    metadata::Metadata,
    semver::Semver,
    ANISE_VERSION,
};
use crate::{errors::IntegrityError, prelude::AniseError, NaifId};
use core::fmt;
use core::marker::PhantomData;
use der::{asn1::OctetStringRef, Decode, Encode, Reader, Writer};
use log::{error, trace};
use std::ops::Deref;

macro_rules! io_imports {
    () => {
        use std::fs::File;
        use std::io::Write;
        use std::path::Path;
        use std::path::PathBuf;
    };
}

io_imports!();

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum DataSetType {
    /// Used only if not encoding a dataset but some other structure
    NotApplicable,
    SpacecraftData,
    PlanetaryData,
}

impl From<u8> for DataSetType {
    fn from(val: u8) -> Self {
        match val {
            0 => DataSetType::NotApplicable,
            1 => DataSetType::SpacecraftData,
            2 => DataSetType::PlanetaryData,
            _ => panic!("Invalid value for DataSetType {val}"),
        }
    }
}

impl From<DataSetType> for u8 {
    fn from(val: DataSetType) -> Self {
        val as u8
    }
}

impl Encode for DataSetType {
    fn encoded_len(&self) -> der::Result<der::Length> {
        (*self as u8).encoded_len()
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        (*self as u8).encode(encoder)
    }
}

impl<'a> Decode<'a> for DataSetType {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let asu8: u8 = decoder.decode()?;
        Ok(Self::from(asu8))
    }
}

/// The kind of data that can be encoded in a dataset
pub trait DataSetT<'a>: Encode + Decode<'a> {}

/// A DataSet is the core structure shared by all ANISE binary data.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DataSet<'a, T: DataSetT<'a>, const ENTRIES: usize> {
    pub metadata: Metadata<'a>,
    /// All datasets have LookUpTable (LUT) that stores the mapping between a key and its index in the ephemeris list.
    pub lut: LookUpTable<'a, ENTRIES>,
    pub data_checksum: u32,
    /// The actual data from the dataset
    pub bytes: &'a [u8],
    _daf_type: PhantomData<T>,
}

/// Dataset builder allows building a dataset. It requires allocations.
#[derive(Clone, Default, Debug)]
pub struct DataSetBuilder<'a, T: DataSetT<'a>, const ENTRIES: usize> {
    pub dataset: DataSet<'a, T, ENTRIES>,
}

impl<'a, T: DataSetT<'a>, const ENTRIES: usize> DataSetBuilder<'a, T, ENTRIES> {
    pub fn push_into(
        &mut self,
        buf: &mut Vec<u8>,
        data: T,
        id: Option<NaifId>,
        name: Option<&'a str>,
    ) -> Result<(), AniseError> {
        let mut this_buf = vec![];
        data.encode_to_vec(&mut this_buf).unwrap();
        // Build this entry data.
        let entry = Entry {
            start_idx: buf.len() as u32,
            end_idx: (buf.len() + this_buf.len()) as u32,
        };

        if id.is_some() && name.is_some() {
            self.dataset.lut.append(id.unwrap(), name.unwrap(), entry)?;
        } else if id.is_some() {
            self.dataset.lut.append_id(id.unwrap(), entry)?;
        } else if name.is_some() {
            self.dataset.lut.append_name(name.unwrap(), entry)?;
        } else {
            return Err(AniseError::ItemNotFound);
        }
        buf.extend_from_slice(&this_buf);

        Ok(())
    }

    pub fn finalize(mut self, buf: &'a [u8]) -> Result<DataSet<'a, T, ENTRIES>, AniseError> {
        self.dataset.bytes = buf;
        self.dataset.set_crc32();
        Ok(self.dataset)
    }
}

impl<'a, T: DataSetT<'a>, const ENTRIES: usize> DataSet<'a, T, ENTRIES> {
    /// Try to load an Anise file from a pointer of bytes
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, AniseError> {
        match Self::from_der(bytes) {
            Ok(ctx) => {
                trace!("[try_from_bytes] loaded context successfully");
                // Check the full integrity on load of the file.
                // TODO: Raise this error
                ctx.check_integrity().unwrap();
                Ok(ctx)
            }
            Err(e) => {
                // If we can't load the file, let's try to load the version only to be helpful
                match bytes.get(0..5) {
                    Some(semver_bytes) => match Semver::from_der(semver_bytes) {
                        Ok(file_version) => {
                            if file_version == ANISE_VERSION {
                                error!("[try_from_bytes] context bytes corrupted but ANISE library version match");
                                Err(AniseError::DecodingError(e))
                            } else {
                                error!(
                                    "[try_from_bytes] context bytes and ANISE library version mismatch"
                                );
                                Err(AniseError::IncompatibleVersion {
                                    got: file_version,
                                    exp: ANISE_VERSION,
                                })
                            }
                        }
                        Err(e) => {
                            error!("[try_from_bytes] context bytes not in ANISE format");
                            Err(AniseError::DecodingError(e))
                        }
                    },
                    None => {
                        error!("[try_from_bytes] context bytes way too short (less than 5 bytes)");
                        Err(AniseError::DecodingError(e))
                    }
                }
            }
        }
    }

    /// Forces to load an Anise file from a pointer of bytes.
    /// **Panics** if the bytes cannot be interpreted as an Anise file.
    pub fn from_bytes<B: Deref<Target = [u8]>>(buf: &'a B) -> Self {
        Self::try_from_bytes(buf).unwrap()
    }

    /// Compute the CRC32 of the underlying bytes
    pub fn crc32(&self) -> u32 {
        crc32fast::hash(self.bytes)
    }

    /// Sets the checksum of this data.
    pub fn set_crc32(&mut self) {
        self.data_checksum = self.crc32();
    }

    pub fn check_integrity(&self) -> Result<(), IntegrityError> {
        // Ensure that the data is correctly decoded
        let computed_chksum = crc32fast::hash(self.bytes);
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

    pub fn get_by_id(&self, id: NaifId) -> Result<T, AniseError> {
        if let Some(entry) = self.lut.by_id.get(&id) {
            // Found the ID
            match T::from_der(&self.bytes[entry.as_range()]) {
                Ok(data) => Ok(data),
                Err(e) => {
                    println!("{e:?}");
                    dbg!(&self.bytes[entry.as_range()]);
                    Err(AniseError::MalformedData(entry.start_idx as usize))
                }
            }
        } else {
            Err(AniseError::ItemNotFound)
        }
    }

    pub fn get_by_name(&self, id: NaifId) -> Result<T, AniseError> {
        if let Some(entry) = self.lut.by_id.get(&id) {
            // Found the ID
            if let Ok(data) = T::from_der(&self.bytes[entry.as_range()]) {
                Ok(data)
            } else {
                Err(AniseError::MalformedData(entry.start_idx as usize))
            }
        } else {
            Err(AniseError::ItemNotFound)
        }
    }

    /// Saves this dataset to the provided file
    /// If overwrite is set to false, and the filename already exists, this function will return an error.

    pub fn save_as(&self, filename: PathBuf, overwrite: bool) -> Result<(), AniseError> {
        use log::{info, warn};

        if Path::new(&filename).exists() {
            if !overwrite {
                return Err(AniseError::FileExists);
            } else {
                warn!("[save_as] overwriting {}", filename.display());
            }
        }

        let mut buf = vec![];

        match File::create(&filename) {
            Ok(mut file) => {
                if let Err(e) = self.encode_to_vec(&mut buf) {
                    return Err(AniseError::DecodingError(e));
                }
                if let Err(e) = file.write_all(&buf) {
                    Err(e.kind().into())
                } else {
                    info!("[OK] dataset saved to {}", filename.display());
                    Ok(())
                }
            }
            Err(e) => Err(e.kind().into()),
        }
    }
}

impl<'a, T: DataSetT<'a>, const ENTRIES: usize> Encode for DataSet<'a, T, ENTRIES> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let as_byte_ref = OctetStringRef::new(self.bytes)?;
        self.metadata.encoded_len()?
            + self.lut.encoded_len()?
            + self.data_checksum.encoded_len()?
            + as_byte_ref.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        let as_byte_ref = OctetStringRef::new(self.bytes)?;
        self.metadata.encode(encoder)?;
        self.lut.encode(encoder)?;
        self.data_checksum.encode(encoder)?;
        as_byte_ref.encode(encoder)
    }
}

impl<'a, T: DataSetT<'a>, const ENTRIES: usize> Decode<'a> for DataSet<'a, T, ENTRIES> {
    fn decode<D: Reader<'a>>(decoder: &mut D) -> der::Result<Self> {
        let metadata = decoder.decode()?;
        let lut = decoder.decode()?;
        let crc32_checksum = decoder.decode()?;
        let bytes: OctetStringRef = decoder.decode()?;
        Ok(Self {
            metadata,
            lut,
            data_checksum: crc32_checksum,
            bytes: bytes.as_bytes(),
            _daf_type: PhantomData::<T>,
        })
    }
}

impl<'a, T: DataSetT<'a>, const ENTRIES: usize> fmt::Display for DataSet<'a, T, ENTRIES> {
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
    };

    use super::{DataSet, Decode, Encode, LookUpTable};

    #[test]
    fn zero_repr() {
        // For this test, we want a data set with zero entries allowed in the LUT.
        let repr = DataSet::<SpacecraftData, 2>::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();
        dbg!(buf.len());

        let repr_dec = DataSet::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        dbg!(core::mem::size_of::<DataSet<SpacecraftData, 2>>());
        dbg!(core::mem::size_of::<DataSet<SpacecraftData, 128>>());
    }

    #[test]
    fn spacecraft_constants_lookup() {
        // Build some data first.
        let full_sc = SpacecraftData {
            name: "full spacecraft",
            comments: "this is an example of encoding spacecraft data",
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
            name: "SRP only spacecraft",
            comments: "this is an example of encoding spacecraft data",
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
        let mut dataset = DataSet::default();
        dataset.lut = lut;
        dataset.bytes = &packed_buf;
        dataset.set_crc32();
        // And encode it.

        let mut buf = vec![];
        dataset.encode_to_vec(&mut buf).unwrap();

        dbg!(buf.len());

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
            name: "full spacecraft",
            comments: "this is an example of encoding spacecraft data",
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
            name: "SRP only spacecraft",
            comments: "this is an example of encoding spacecraft data",
            srp_data: Some(SRPData::default()),
            ..Default::default()
        };

        // Initialize the overall buffer for building the data
        let mut buf = vec![];
        let mut builder = DataSetBuilder::default();
        builder
            .push_into(&mut buf, srp_sc, Some(-20), Some("SRP spacecraft"))
            .unwrap();

        builder
            .push_into(&mut buf, full_sc, Some(-50), Some("Full spacecraft"))
            .unwrap();

        // Pushing without name as ID -51
        builder
            .push_into(&mut buf, full_sc, Some(-51), None)
            .unwrap();

        // Pushing without ID
        builder
            .push_into(&mut buf, srp_sc, None, Some("ID less SRP spacecraft"))
            .unwrap();

        let dataset = builder.finalize(&buf).unwrap();

        // And encode it.

        let mut ebuf = vec![];
        dataset.encode_to_vec(&mut ebuf).unwrap();

        dbg!(ebuf.len());

        let repr_dec = DataSet::<SpacecraftData, 16>::from_bytes(&ebuf);

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
}
