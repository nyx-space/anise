/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use crate::prelude::{Aberration, Frame};
use crate::NaifId;

pub use crate::analysis::elements::OrbitalElement;
use crate::analysis::specs::{OrthogonalFrame, Plane};

use super::prelude::{ScalarExpr, VectorExpr};
use super::specs::{FrameSpec, StateSpec};

/// ScalarExpr defines a scalar computation from a (set of) vector expression(s).
#[pyclass]
#[pyo3(module = "anise.analysis", name = "ScalarExpr", get_all, set_all)]
pub enum PyScalarExpr {
    Constant(f64),
    /// Mean radius of the provided body, must be loaded in the almanac
    MeanEquatorialRadius {
        celestial_object: i32,
    },
    SemiMajorEquatorialRadius {
        celestial_object: i32,
    },
    SemiMinorEquatorialRadius {
        celestial_object: i32,
    },
    PolarRadius {
        celestial_object: i32,
    },
    Flattening {
        celestial_object: i32,
    },
    GravParam {
        celestial_object: i32,
    },
    Add {
        a: Py<PyScalarExpr>,
        b: Py<PyScalarExpr>,
    },
    Mul {
        a: Py<PyScalarExpr>,
        b: Py<PyScalarExpr>,
    },
    Negate(Py<PyScalarExpr>),
    Invert(Py<PyScalarExpr>),
    Sqrt(Py<PyScalarExpr>),
    Powi {
        scalar: Py<PyScalarExpr>,
        n: i32,
    },
    Powf {
        scalar: Py<PyScalarExpr>,
        n: f64,
    },
    Cos(Py<PyScalarExpr>),
    Sin(Py<PyScalarExpr>),
    Tan(Py<PyScalarExpr>),
    /// Compute the arccos, returned in degrees
    Acos(Py<PyScalarExpr>),
    /// Compute the arcsin, returned in degrees
    Asin(Py<PyScalarExpr>),
    /// Compute the arctan2 (i.e. arctan with quadrant check), returned in degrees
    Atan2 {
        y: Py<PyScalarExpr>,
        x: Py<PyScalarExpr>,
    },
    /// Computes v % m
    Modulo {
        v: Py<PyScalarExpr>,
        m: Py<PyScalarExpr>,
    },
    Norm(Py<PyVectorExpr>),
    NormSquared(Py<PyVectorExpr>),
    DotProduct {
        a: Py<PyVectorExpr>,
        b: Py<PyVectorExpr>,
    },
    /// Angle between two vectors, in degrees
    AngleBetween {
        a: Py<PyVectorExpr>,
        b: Py<PyVectorExpr>,
    },
    VectorX(Py<PyVectorExpr>),
    VectorY(Py<PyVectorExpr>),
    VectorZ(Py<PyVectorExpr>),
    Element(OrbitalElement),
    /// Computes the eclipsing percentage due to the eclipsing frame. Aberration correction is that of the state spec.
    SolarEclipsePercentage {
        eclipsing_frame: Frame,
    },
    /// Computes the occultation percentage of the back_frame due to the front_frame. Aberration correction is that of the state spec.
    OccultationPercentage {
        back_frame: Frame,
        front_frame: Frame,
    },
    /// Computes the beta angle, in degrees. Aberration correction is that of the state spec.
    BetaAngle(),
    /// Compute the local solar time, in hours
    LocalSolarTime(),
    /// Computes the local time of the ascending node, in hours
    LocalTimeAscNode(),
    /// Computes the local time of the descending node, in hours
    LocalTimeDescNode(),
    /// Computes the Sun angle where observer_id is the ID of the spacecraft for example.
    /// If the frame of the state spec is in an Earth frame, then this computes the Sun Probe Earth angle.
    /// Refer to the sun_angle_deg function for detailed documentation.
    SunAngle {
        observer_id: NaifId,
    },
    AzimuthFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
    ElevationFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
    RangeFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
    RangeRateFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
}

/* #[pymethods]
impl PyScalarExpr {
    fn evaluate(
        &self,
        orbit: Orbit,
        ab_corr: Option<Aberration>,
        almanac: &Almanac,
    ) -> Result<f64, PyErr> {
        let scalar: ScalarExpr = self.try_into()?;

        scalar
            .evaluate(orbit, ab_corr, almanac)
            .map_err(|e| PyException::new_err(e.to_string()))
    }
} */

#[pyclass]
#[pyo3(module = "anise.analysis", name = "VectorExpr", get_all, set_all)]
pub enum PyVectorExpr {
    // Vector with unspecified units, for arbitrary computations
    Fixed {
        x: f64,
        y: f64,
        z: f64,
    },
    /// Radius/position vector of this state specification
    Radius(Py<PyStateSpec>),
    /// Velocity vector of this state specification
    Velocity(Py<PyStateSpec>),
    /// Orbital moment (H) vector of this state specification
    OrbitalMomentum(Py<PyStateSpec>),
    /// Eccentricity vector of this state specification
    EccentricityVector(Py<PyStateSpec>),
    /// Cross product between two vector expression
    CrossProduct {
        a: Py<PyVectorExpr>,
        b: Py<PyVectorExpr>,
    },
    /// Unit vector of this vector expression, returns zero vector if norm less than 1e-12
    Unit(Py<PyVectorExpr>),
    /// Negate a vector
    /// /// Negate a vector.
    Negate(Py<PyVectorExpr>),
    /// Vector projection of a onto b
    VecProjection {
        a: Py<PyVectorExpr>,
        b: Py<PyVectorExpr>,
    },
    // This should be as simple as multiplying the input VectorExpr by the DCM.
    // I think it makes sense to have trivial rotations like VNC, RIC, RCN available in the frame spec.
    // The test should consist in checking that we can rebuild the VNC frame and project the Sun Earth vector onto
    // the VNC frame of that same Sun Earth orbit, returning the X, Y, or Z component.
    // Projection should allow XY, XZ, YZ which determines the components to account for.
    /// Multiplies the DCM of thr frame with this vector, thereby rotating it into the provided orthogonal frame, optionally projecting onto the plan, optionally projecting onto the plane
    Project {
        v: Py<PyVectorExpr>,
        frame: Py<PyOrthogonalFrame>,
        plane: Option<Plane>,
    },
}

/// StateSpec allows defining a state from the target to the observer
#[pyclass]
#[pyo3(module = "anise.analysis", name = "StateSpec", get_all, set_all)]
pub struct PyStateSpec {
    pub target_frame: PyFrameSpec,
    pub observer_frame: PyFrameSpec,
    pub ab_corr: Option<Aberration>,
}

#[pymethods]
impl PyStateSpec {
    #[new]
    fn new(
        target_frame: PyFrameSpec,
        observer_frame: PyFrameSpec,
        ab_corr: Option<Aberration>,
    ) -> Self {
        Self {
            target_frame,
            observer_frame,
            ab_corr,
        }
    }
}

/// FrameSpec allows defining a frame that can be computed from another set of loaded frames, which include a center.
#[pyclass]
#[pyo3(module = "anise.analysis", name = "FrameSpec", get_all, set_all)]
pub enum PyFrameSpec {
    Loaded(Frame),
    Manual {
        name: String,
        defn: Py<PyOrthogonalFrame>,
    },
}

// Manual implementation of Clone to handle the Py<T> field correctly.
impl Clone for PyFrameSpec {
    fn clone(&self) -> Self {
        // To clone a Python object reference (Py<T>), we must acquire the GIL.
        Python::attach(|py| -> PyFrameSpec {
            match self {
                // The Loaded variant is simple, as Frame is already Clone.
                PyFrameSpec::Loaded(frame) => PyFrameSpec::Loaded(*frame),

                // For the Manual variant, we clone each field individually.
                PyFrameSpec::Manual { name, defn } => {
                    PyFrameSpec::Manual {
                        name: name.clone(),
                        // The clone_ref() method on Py<T> requires the GIL token (`py`),
                        // which we have here. This is the correct way to clone a Py<T>.
                        defn: defn.clone_ref(py),
                    }
                }
            }
        })
    }
}

// Defines how to build an orthogonal frame from custom vector expressions
//
// WARNING: Building such a frame does NOT normalize the vectors, you must use the Unit vector expression
// to build an orthonormal frame.
#[pyclass]
#[pyo3(module = "anise.analysis", name = "OrthogonalFrame", get_all, set_all)]
pub enum PyOrthogonalFrame {
    XY {
        x: Py<PyVectorExpr>,
        y: Py<PyVectorExpr>,
    },
    XZ {
        x: Py<PyVectorExpr>,
        z: Py<PyVectorExpr>,
    },
    YZ {
        y: Py<PyVectorExpr>,
        z: Py<PyVectorExpr>,
    },
}

// *** Implement the From<RustType> for PythonType to convert the LISP representation ***//

impl TryFrom<ScalarExpr> for PyScalarExpr {
    type Error = PyErr;

    fn try_from(value: ScalarExpr) -> Result<Self, Self::Error> {
        match value {
            ScalarExpr::BetaAngle => Ok(Self::BetaAngle()),
            ScalarExpr::LocalSolarTime => Ok(Self::LocalSolarTime()),
            ScalarExpr::LocalTimeAscNode => Ok(Self::LocalTimeAscNode()),
            ScalarExpr::LocalTimeDescNode => Ok(Self::LocalTimeDescNode()),
            ScalarExpr::Constant(v) => Ok(Self::Constant(v)),
            ScalarExpr::SunAngle { observer_id } => Ok(Self::SunAngle { observer_id }),
            ScalarExpr::AzimuthFromLocation {
                location_id,
                obstructing_body,
            } => Ok(Self::AzimuthFromLocation {
                location_id,
                obstructing_body,
            }),
            ScalarExpr::ElevationFromLocation {
                location_id,
                obstructing_body,
            } => Ok(Self::ElevationFromLocation {
                location_id,
                obstructing_body,
            }),
            ScalarExpr::RangeFromLocation {
                location_id,
                obstructing_body,
            } => Ok(Self::RangeFromLocation {
                location_id,
                obstructing_body,
            }),
            ScalarExpr::RangeRateFromLocation {
                location_id,
                obstructing_body,
            } => Ok(Self::RangeRateFromLocation {
                location_id,
                obstructing_body,
            }),
            ScalarExpr::SolarEclipsePercentage { eclipsing_frame } => {
                Ok(Self::SolarEclipsePercentage { eclipsing_frame })
            }
            ScalarExpr::OccultationPercentage {
                back_frame,
                front_frame,
            } => Ok(Self::OccultationPercentage {
                back_frame,
                front_frame,
            }),
            ScalarExpr::Element(e) => Ok(Self::Element(e)),
            ScalarExpr::MeanEquatorialRadius { celestial_object } => {
                Ok(Self::MeanEquatorialRadius { celestial_object })
            }
            ScalarExpr::SemiMajorEquatorialRadius { celestial_object } => {
                Ok(Self::SemiMajorEquatorialRadius { celestial_object })
            }
            ScalarExpr::SemiMinorEquatorialRadius { celestial_object } => {
                Ok(Self::SemiMinorEquatorialRadius { celestial_object })
            }
            ScalarExpr::PolarRadius { celestial_object } => {
                Ok(Self::PolarRadius { celestial_object })
            }
            ScalarExpr::Flattening { celestial_object } => {
                Ok(Self::Flattening { celestial_object })
            }
            ScalarExpr::GravParam { celestial_object } => Ok(Self::GravParam { celestial_object }),
            ScalarExpr::Norm(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Norm(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?))
            }),
            ScalarExpr::NormSquared(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::NormSquared(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?))
            }),
            ScalarExpr::VectorX(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::VectorX(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?))
            }),
            ScalarExpr::VectorY(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::VectorY(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?))
            }),
            ScalarExpr::VectorZ(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::VectorZ(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?))
            }),
            ScalarExpr::DotProduct { a, b } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::DotProduct {
                    a: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(a)?)?,
                    b: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(b)?)?,
                })
            }),
            ScalarExpr::AngleBetween { a, b } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::AngleBetween {
                    a: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(a)?)?,
                    b: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(b)?)?,
                })
            }),
            ScalarExpr::Negate(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Negate(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Invert(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Invert(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Cos(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Cos(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Sin(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Sin(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Tan(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Tan(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Acos(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Acos(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Asin(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Asin(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Sqrt(v) => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Sqrt(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?))
            }),
            ScalarExpr::Powi { scalar, n } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Powi {
                    scalar: Py::new(
                        py,
                        <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*scalar)?,
                    )?,
                    n,
                })
            }),
            ScalarExpr::Powf { scalar, n } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Powf {
                    scalar: Py::new(
                        py,
                        <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*scalar)?,
                    )?,
                    n,
                })
            }),
            ScalarExpr::Add { a, b } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Add {
                    a: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*a)?)?,
                    b: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*b)?)?,
                })
            }),
            ScalarExpr::Mul { a, b } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Mul {
                    a: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*a)?)?,
                    b: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*b)?)?,
                })
            }),
            ScalarExpr::Atan2 { y, x } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Atan2 {
                    y: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*y)?)?,
                    x: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*x)?)?,
                })
            }),
            ScalarExpr::Modulo { v, m } => Python::attach(|py| -> Result<Self, PyErr> {
                Ok(Self::Modulo {
                    v: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?)?,
                    m: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*m)?)?,
                })
            }),
        }
    }
}

impl TryFrom<OrthogonalFrame> for PyOrthogonalFrame {
    type Error = PyErr;
    fn try_from(value: OrthogonalFrame) -> Result<Self, PyErr> {
        Python::attach(|py| -> Result<Self, PyErr> {
            match value {
                OrthogonalFrame::XY { x, y } => Ok(Self::XY {
                    x: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(x)?)?,
                    y: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(y)?)?,
                }),
                OrthogonalFrame::XZ { x, z } => Ok(Self::XZ {
                    x: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(x)?)?,
                    z: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(z)?)?,
                }),
                OrthogonalFrame::YZ { y, z } => Ok(Self::YZ {
                    y: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(y)?)?,
                    z: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(z)?)?,
                }),
            }
        })
    }
}

impl TryFrom<VectorExpr> for PyVectorExpr {
    type Error = PyErr;
    fn try_from(value: VectorExpr) -> Result<Self, PyErr> {
        Python::attach(|py| -> Result<Self, PyErr> {
            match value {
                VectorExpr::Fixed { x, y, z } => Ok(Self::Fixed { x, y, z }),
                VectorExpr::CrossProduct { a, b } => Ok(Self::CrossProduct {
                    a: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(*a)?)?,
                    b: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(*b)?)?,
                }),
                VectorExpr::VecProjection { a, b } => Ok(Self::VecProjection {
                    a: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(*a)?)?,
                    b: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(*b)?)?,
                }),
                VectorExpr::Unit(v) => Ok(Self::Unit(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(*v)?,
                )?)),
                VectorExpr::Negate(v) => Ok(Self::Negate(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(*v)?,
                )?)),
                VectorExpr::Project { v, frame, plane } => Ok(Self::Project {
                    v: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(*v)?)?,
                    frame: Py::new(
                        py,
                        <OrthogonalFrame as TryInto<PyOrthogonalFrame>>::try_into(*frame)?,
                    )?,
                    plane,
                }),
                VectorExpr::Radius(spec) => Ok(Self::Radius(Py::new(
                    py,
                    <StateSpec as TryInto<PyStateSpec>>::try_into(spec)?,
                )?)),
                VectorExpr::Velocity(spec) => Ok(Self::Velocity(Py::new(
                    py,
                    <StateSpec as TryInto<PyStateSpec>>::try_into(spec)?,
                )?)),
                VectorExpr::EccentricityVector(spec) => Ok(Self::EccentricityVector(Py::new(
                    py,
                    <StateSpec as TryInto<PyStateSpec>>::try_into(spec)?,
                )?)),
                VectorExpr::OrbitalMomentum(spec) => Ok(Self::OrbitalMomentum(Py::new(
                    py,
                    <StateSpec as TryInto<PyStateSpec>>::try_into(spec)?,
                )?)),
            }
        })
    }
}
impl TryFrom<StateSpec> for PyStateSpec {
    type Error = PyErr;
    fn try_from(value: StateSpec) -> Result<Self, PyErr> {
        Ok(PyStateSpec {
            target_frame: value.target_frame.try_into()?,
            observer_frame: value.observer_frame.try_into()?,
            ab_corr: value.ab_corr,
        })
    }
}
impl TryFrom<FrameSpec> for PyFrameSpec {
    type Error = PyErr;
    fn try_from(value: FrameSpec) -> Result<Self, PyErr> {
        Python::attach(|py| -> Result<Self, PyErr> {
            Ok(match value {
                FrameSpec::Loaded(f) => PyFrameSpec::Loaded(f),
                FrameSpec::Manual { name, defn } => PyFrameSpec::Manual {
                    name,
                    defn: Py::new(
                        py,
                        <OrthogonalFrame as TryInto<PyOrthogonalFrame>>::try_into(*defn)?,
                    )?,
                },
            })
        })
    }
}
