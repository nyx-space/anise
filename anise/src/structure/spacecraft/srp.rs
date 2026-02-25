/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Writer};
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::{PyBytes, PyType};

#[cfg_attr(feature = "python", pyclass(get_all, set_all, module = "anise.astro"))]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SRPData {
    /// Solar radiation pressure area in m^2 -- default 0.0
    /// :rtype: float
    pub area_m2: f64,
    /// Solar radiation pressure coefficient of reflectivity (C_r) -- default 1.8
    /// :rtype: float
    pub coeff_reflectivity: f64,
}

impl SRPData {
    pub fn from_area(area_m2: f64) -> Self {
        Self {
            area_m2,
            ..Default::default()
        }
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl SRPData {
    #[new]
    #[pyo3(signature = (area_m2, coeff_reflectivity = None))]
    fn py_new(area_m2: f64, coeff_reflectivity: Option<f64>) -> Self {
        Self {
            area_m2,
            coeff_reflectivity: coeff_reflectivity.unwrap_or(Self::default().coeff_reflectivity),
        }
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    fn __repr__(&self) -> String {
        format!("{self:?} @ {self:p}")
    }

    /// Decodes an ASN.1 DER encoded byte array into an SRPData object.
    ///
    /// :type data: bytes
    /// :rtype: SRPData
    #[classmethod]
    pub fn from_asn1(_cls: &Bound<'_, PyType>, data: &[u8]) -> PyResult<Self> {
        match Self::from_der(data) {
            Ok(obj) => Ok(obj),
            Err(e) => Err(PyValueError::new_err(format!("ASN.1 decoding error: {e}"))),
        }
    }

    /// Encodes this SRPData object into an ASN.1 DER encoded byte array.
    ///
    /// :rtype: bytes
    pub fn to_asn1<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let mut buf = Vec::new();
        match self.encode_to_vec(&mut buf) {
            Ok(_) => Ok(PyBytes::new(py, &buf)),
            Err(e) => Err(PyValueError::new_err(format!("ASN.1 encoding error: {e}"))),
        }
    }
}

impl Default for SRPData {
    fn default() -> Self {
        Self {
            area_m2: 0.0,
            coeff_reflectivity: 1.8,
        }
    }
}

impl Encode for SRPData {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.area_m2.encoded_len()? + self.coeff_reflectivity.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.area_m2.encode(encoder)?;
        self.coeff_reflectivity.encode(encoder)
    }
}

impl<'a> Decode<'a> for SRPData {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            area_m2: decoder.decode()?,
            coeff_reflectivity: decoder.decode()?,
        })
    }
}

#[cfg(test)]
mod srp_ut {
    use super::{Decode, Encode, SRPData};
    #[test]
    fn zero_repr() {
        let repr = SRPData {
            area_m2: Default::default(),
            coeff_reflectivity: Default::default(),
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SRPData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn default_repr() {
        let repr = SRPData::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SRPData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
