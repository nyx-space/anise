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
};
use crate::{errors::IntegrityErrorKind, prelude::AniseError, NaifId};
use der::{asn1::OctetStringRef, Decode, Encode, Reader, Writer};
use log::error;

/// A DataSet is the core structure shared by all ANISE binary data.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DataSet<'a> {
    pub metadata: Metadata<'a>,
    /// All datasets have LookUpTable (LUT) that stores the mapping between a key and its index in the ephemeris list.
    pub lut: LookUpTable<'a>,
    pub data_checksum: u32,
    /// The actual data from the dataset
    pub bytes: &'a [u8],
}

/// Dataset builder allows building a dataset. It requires allocations.
#[derive(Clone, Default, Debug)]
pub struct DataSetBuilder<'a> {
    pub dataset: DataSet<'a>,
}

impl<'a> DataSetBuilder<'a> {
    pub fn push_into<T: Encode>(
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

    pub fn finalize(mut self, buf: &'a [u8]) -> Result<DataSet<'a>, AniseError> {
        self.dataset.bytes = buf;
        self.dataset.set_crc32();
        Ok(self.dataset)
    }
}

impl<'a> DataSet<'a> {
    /// Compute the CRC32 of the underlying bytes
    pub fn crc32(&self) -> u32 {
        crc32fast::hash(self.bytes)
    }

    /// Sets the checksum of this data.
    pub fn set_crc32(&mut self) {
        self.data_checksum = self.crc32();
    }

    pub fn check_integrity(&self) -> Result<(), AniseError> {
        // Ensure that the data is correctly decoded
        let computed_chksum = crc32fast::hash(self.bytes);
        if computed_chksum == self.data_checksum {
            Ok(())
        } else {
            error!(
                "[integrity] expected hash {} but computed {}",
                self.data_checksum, computed_chksum
            );
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::ChecksumInvalid {
                    expected: self.data_checksum,
                    computed: computed_chksum,
                },
            ))
        }
    }

    /// Scrubs the data by computing the CRC32 of the bytes and making sure that it still matches the previously known hash
    pub fn scrub(&self) -> Result<(), AniseError> {
        if self.crc32() == self.data_checksum {
            Ok(())
        } else {
            // Compiler will optimize the double computation away
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::ChecksumInvalid {
                    expected: self.data_checksum,
                    computed: self.crc32(),
                },
            ))
        }
    }

    pub fn get_by_id<T: Decode<'a>>(&self, id: NaifId) -> Result<T, AniseError> {
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

    pub fn get_by_name<T: Decode<'a>>(&self, id: NaifId) -> Result<T, AniseError> {
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
}

impl<'a> Encode for DataSet<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let as_byte_ref = OctetStringRef::new(self.bytes)?;
        self.metadata.encoded_len()?
            + self.lut.encoded_len()?
            + self.data_checksum.encoded_len()?
            + as_byte_ref.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let as_byte_ref = OctetStringRef::new(self.bytes)?;
        self.metadata.encode(encoder)?;
        self.lut.encode(encoder)?;
        self.data_checksum.encode(encoder)?;
        as_byte_ref.encode(encoder)
    }
}

impl<'a> Decode<'a> for DataSet<'a> {
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
        })
    }
}

#[cfg(test)]
mod dataset_ut {
    use crate::structure::{
        dataset::DataSetBuilder,
        lookuptable::Entry,
        spacecraft::{DragData, Inertia, Mass, SRPData, SpacecraftConstants},
    };

    use super::{DataSet, Decode, Encode, LookUpTable};

    #[test]
    fn zero_repr() {
        let repr = DataSet::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();
        dbg!(buf.len());

        let repr_dec = DataSet::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        dbg!(core::mem::size_of::<DataSet>());
    }

    #[test]
    fn spacecraft_constants_lookup() {
        // Build some data first.
        let full_sc = SpacecraftConstants {
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
        let srp_sc = SpacecraftConstants {
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
        let full_sc_dec =
            SpacecraftConstants::from_der(&packed_buf[full_sc_entry.as_range()]).unwrap();
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
        let srp_sc_dec =
            SpacecraftConstants::from_der(&packed_buf[srp_sc_entry.as_range()]).unwrap();
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

        let repr_dec = DataSet::from_der(&buf).unwrap();

        assert_eq!(dataset, repr_dec);

        assert!(repr_dec.check_integrity().is_ok());

        // Now that the data is valid, let's fetch the data back

        let full_sc_repr = repr_dec.get_by_id::<SpacecraftConstants>(-50).unwrap();
        assert_eq!(full_sc_repr, full_sc);

        let srp_repr = repr_dec.get_by_id::<SpacecraftConstants>(-20).unwrap();
        assert_eq!(srp_repr, srp_sc);

        // And check that we get an error if the data is wrong.
        assert!(repr_dec.get_by_id::<SpacecraftConstants>(0).is_err())
    }

    #[test]
    fn spacecraft_constants_lookup_builder() {
        // Build some data first.
        let full_sc = SpacecraftConstants {
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
        let srp_sc = SpacecraftConstants {
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

        let repr_dec = DataSet::from_der(&ebuf).unwrap();

        assert_eq!(dataset, repr_dec);

        assert!(repr_dec.check_integrity().is_ok());

        // Now that the data is valid, let's fetch the data back

        let full_sc_repr = repr_dec.get_by_id::<SpacecraftConstants>(-50).unwrap();
        assert_eq!(full_sc_repr, full_sc);

        let srp_repr = repr_dec.get_by_id::<SpacecraftConstants>(-20).unwrap();
        assert_eq!(srp_repr, srp_sc);

        // And check that we get an error if the data is wrong.
        assert!(repr_dec.get_by_id::<SpacecraftConstants>(0).is_err())
    }
}
