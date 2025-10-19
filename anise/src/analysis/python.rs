/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use hifitime::{Duration, Epoch, TimeSeries};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::collections::HashMap;

use crate::prelude::{Aberration, Almanac, Frame, Orbit};
use crate::NaifId;

pub use crate::analysis::elements::OrbitalElement;
use crate::analysis::specs::{OrthogonalFrame, Plane};

use super::event::{Event, EventArc, EventDetails};
use super::prelude::{ScalarExpr, VectorExpr};
use super::specs::{FrameSpec, StateSpec};
use super::{AnalysisError, ReportScalars};

#[pymethods]
impl Almanac {
    /// Report a set of scalar expressions, optionally with aliases, at a fixed time step defined in the TimeSeries.
    ///
    /// :type report: ReportScalars
    /// :type time_series: TimeSeries
    /// :rtype: dict
    #[pyo3(name = "report_scalars")]
    pub fn py_report_scalars(
        &self,
        py: Python,
        report: &ReportScalars,
        time_series: TimeSeries,
    ) -> Result<HashMap<String, HashMap<String, f64>>, AnalysisError> {
        let data = py.detach(|| self.report_scalars(report, time_series));
        // Modify the values to set errors to NaN
        let mut rslt = HashMap::new();
        for (k, v) in data {
            let mut data_epoch_ok = HashMap::new();
            for (col, value) in v? {
                data_epoch_ok.insert(col, value.unwrap_or(f64::NAN));
            }
            rslt.insert(k.to_isoformat(), data_epoch_ok);
        }

        Ok(rslt)
    }

    /// Report all of the states and event details where the provided event occurs.
    ///
    /// # Limitations
    /// This method uses a Brent solver, provides a superlinearity convergence (Golden ratio rate).
    /// If the function that defines the event is not unimodal, the event finder may not converge correctly.
    /// After the Brent solver is used, this function will check the median gap between events. Assuming most events are periodic,
    /// any gap whose median repetition is greater than 125% will be slow searches. This _typically_ finds all of the events ... but it
    /// may also add duplicates! To prevent reporting duplicate events, the found events are deduplicated if the same event is found
    /// within 3 times the epoch precision. For example, if the epoch precision is 100 ms, if three events are "found" within 300 ms of each other
    /// then only one of these three is preserved.
    ///
    /// # Heuristic detail
    /// The initial search step is 1% of the duration requested, if the heuristic is set to None.
    /// For example, if the trajectory is 100 days long, then we split the trajectory into 100 chunks of 1 day and see whether
    /// the event is in there. If the event happens twice or more times within 1% of the trajectory duration, only the _one_ of
    /// such events will be found.
    ///
    /// :type state_spec: StateSpec
    /// :type event: Event
    /// :type start_epoch: Epoch
    /// :type end_epoch: Epoch
    /// :type heuristic: Duration, optional
    /// :rtype: list
    #[pyo3(name = "report_events", signature=(state_spec, event, start_epoch, end_epoch, heuristic=None))]
    #[allow(clippy::identity_op)]
    fn py_report_events(
        &self,
        py: Python,
        state_spec: PyStateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
        heuristic: Option<Duration>,
    ) -> Result<Vec<EventDetails>, AnalysisError> {
        py.detach(|| {
            self.report_events(
                &StateSpec::from(state_spec),
                event,
                start_epoch,
                end_epoch,
                heuristic,
            )
        })
    }
    /// Slow approach to finding **all** of the events between two epochs. This will evaluate ALL epochs in between the two bounds.
    /// This approach is more robust, but more computationally demanding since it's O(N).
    ///
    /// :type state_spec: StateSpec
    /// :type event: Event
    /// :type start_epoch: Epoch
    /// :type end_epoch: Epoch
    /// :rtype: list
    #[pyo3(name = "report_events_slow")]
    #[allow(clippy::identity_op)]
    fn py_report_events_slow(
        &self,
        py: Python,
        state_spec: PyStateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<Vec<EventDetails>, AnalysisError> {
        py.detach(|| {
            self.report_events_slow(&StateSpec::from(state_spec), event, start_epoch, end_epoch)
        })
    }

    /// Find all event arcs, i.e. the start and stop time of when a given event occurs. This function
    /// calls the memory and computationally intensive [report_events_slow] function.
    ///
    /// :type state_spec: StateSpec
    /// :type event: Event
    /// :type start_epoch: Epoch
    /// :type end_epoch: Epoch
    /// :rtype: list
    #[pyo3(name = "report_event_arcs")]
    fn py_report_event_arcs(
        &self,
        py: Python,
        state_spec: PyStateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<Vec<EventArc>, AnalysisError> {
        py.detach(|| {
            self.report_event_arcs(&StateSpec::from(state_spec), event, start_epoch, end_epoch)
        })
    }
}

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

impl Clone for PyScalarExpr {
    fn clone(&self) -> Self {
        Python::attach(|py| -> PyScalarExpr {
            match self {
                Self::Constant(c) => Self::Constant(*c),
                Self::MeanEquatorialRadius { celestial_object } => Self::MeanEquatorialRadius {
                    celestial_object: *celestial_object,
                },
                Self::SemiMajorEquatorialRadius { celestial_object } => {
                    Self::SemiMajorEquatorialRadius {
                        celestial_object: *celestial_object,
                    }
                }
                Self::SemiMinorEquatorialRadius { celestial_object } => {
                    Self::SemiMinorEquatorialRadius {
                        celestial_object: *celestial_object,
                    }
                }
                Self::PolarRadius { celestial_object } => Self::PolarRadius {
                    celestial_object: *celestial_object,
                },
                Self::Flattening { celestial_object } => Self::Flattening {
                    celestial_object: *celestial_object,
                },
                Self::GravParam { celestial_object } => Self::GravParam {
                    celestial_object: *celestial_object,
                },
                Self::Add { a, b } => Self::Add {
                    a: a.clone_ref(py),
                    b: b.clone_ref(py),
                },
                Self::Mul { a, b } => Self::Mul {
                    a: a.clone_ref(py),
                    b: b.clone_ref(py),
                },
                Self::Negate(s) => Self::Negate(s.clone_ref(py)),
                Self::Invert(s) => Self::Invert(s.clone_ref(py)),
                Self::Sqrt(s) => Self::Sqrt(s.clone_ref(py)),
                Self::Powi { scalar, n } => Self::Powi {
                    scalar: scalar.clone_ref(py),
                    n: *n,
                },
                Self::Powf { scalar, n } => Self::Powf {
                    scalar: scalar.clone_ref(py),
                    n: *n,
                },
                Self::Cos(s) => Self::Cos(s.clone_ref(py)),
                Self::Sin(s) => Self::Sin(s.clone_ref(py)),
                Self::Tan(s) => Self::Tan(s.clone_ref(py)),
                Self::Acos(s) => Self::Acos(s.clone_ref(py)),
                Self::Asin(s) => Self::Asin(s.clone_ref(py)),
                Self::Atan2 { y, x } => Self::Atan2 {
                    y: y.clone_ref(py),
                    x: x.clone_ref(py),
                },
                Self::Modulo { v, m } => Self::Modulo {
                    v: v.clone_ref(py),
                    m: m.clone_ref(py),
                },
                Self::Norm(v) => Self::Norm(v.clone_ref(py)),
                Self::NormSquared(v) => Self::NormSquared(v.clone_ref(py)),
                Self::DotProduct { a, b } => Self::DotProduct {
                    a: a.clone_ref(py),
                    b: b.clone_ref(py),
                },
                Self::AngleBetween { a, b } => Self::AngleBetween {
                    a: a.clone_ref(py),
                    b: b.clone_ref(py),
                },
                Self::VectorX(v) => Self::VectorX(v.clone_ref(py)),
                Self::VectorY(v) => Self::VectorY(v.clone_ref(py)),
                Self::VectorZ(v) => Self::VectorZ(v.clone_ref(py)),
                Self::Element(e) => Self::Element(*e),
                Self::SolarEclipsePercentage { eclipsing_frame } => Self::SolarEclipsePercentage {
                    eclipsing_frame: *eclipsing_frame,
                },
                Self::OccultationPercentage {
                    back_frame,
                    front_frame,
                } => Self::OccultationPercentage {
                    back_frame: *back_frame,
                    front_frame: *front_frame,
                },
                Self::BetaAngle() => Self::BetaAngle(),
                Self::LocalSolarTime() => Self::LocalSolarTime(),
                Self::LocalTimeAscNode() => Self::LocalTimeAscNode(),
                Self::LocalTimeDescNode() => Self::LocalTimeDescNode(),
                Self::SunAngle { observer_id } => Self::SunAngle {
                    observer_id: *observer_id,
                },
                Self::AzimuthFromLocation {
                    location_id,
                    obstructing_body,
                } => Self::AzimuthFromLocation {
                    location_id: *location_id,
                    obstructing_body: *obstructing_body,
                },
                Self::ElevationFromLocation {
                    location_id,
                    obstructing_body,
                } => Self::ElevationFromLocation {
                    location_id: *location_id,
                    obstructing_body: *obstructing_body,
                },
                Self::RangeFromLocation {
                    location_id,
                    obstructing_body,
                } => Self::RangeFromLocation {
                    location_id: *location_id,
                    obstructing_body: *obstructing_body,
                },
                Self::RangeRateFromLocation {
                    location_id,
                    obstructing_body,
                } => Self::RangeRateFromLocation {
                    location_id: *location_id,
                    obstructing_body: *obstructing_body,
                },
            }
        })
    }
}

#[pymethods]
impl PyScalarExpr {
    /// Compute this ScalarExpr for the provided Orbit
    ///
    /// :type orbit: Orbit
    /// :type almanac: Almanac
    /// :type ab_corr: Aberration, optional
    /// :rtype:float
    #[pyo3(signature=(orbit, almanac, ab_corr=None))]
    fn evaluate(
        &self,
        orbit: Orbit,
        almanac: &Almanac,
        ab_corr: Option<Aberration>,
    ) -> Result<f64, PyErr> {
        let py_scalar = self.clone();
        let scalar = ScalarExpr::from(py_scalar);

        scalar
            .evaluate(orbit, ab_corr, almanac)
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    /// Convert the S-Expression to a ScalarExpr
    /// :type expr: str
    /// :rtype: ScalarExpr
    #[classmethod]
    fn from_s_expr(_cls: Bound<'_, PyType>, expr: &str) -> Result<Self, PyErr> {
        let scalar =
            ScalarExpr::from_s_expr(expr).map_err(|e| PyException::new_err(e.to_string()))?;

        scalar.try_into()
    }

    /// Converts this ScalarExpr to its S-Expression
    /// :rtype: str
    fn to_s_expr(&self) -> Result<String, PyErr> {
        let scalar = ScalarExpr::from(self.clone());

        scalar
            .to_s_expr()
            .map_err(|e| PyException::new_err(e.to_string()))
    }
}

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
    /// Negate a vector.
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

impl Clone for PyVectorExpr {
    fn clone(&self) -> Self {
        // We must acquire the GIL to safely clone the Py<T> references.
        Python::attach(|py| -> PyVectorExpr {
            match self {
                Self::Fixed { x, y, z } => Self::Fixed {
                    x: *x,
                    y: *y,
                    z: *z,
                },
                Self::Radius(s) => Self::Radius(s.clone_ref(py)),
                Self::Velocity(s) => Self::Velocity(s.clone_ref(py)),
                Self::OrbitalMomentum(s) => Self::OrbitalMomentum(s.clone_ref(py)),
                Self::EccentricityVector(s) => Self::EccentricityVector(s.clone_ref(py)),
                Self::CrossProduct { a, b } => Self::CrossProduct {
                    a: a.clone_ref(py),
                    b: b.clone_ref(py),
                },
                Self::Unit(v) => Self::Unit(v.clone_ref(py)),
                Self::Negate(v) => Self::Negate(v.clone_ref(py)),
                Self::VecProjection { a, b } => Self::VecProjection {
                    a: a.clone_ref(py),
                    b: b.clone_ref(py),
                },
                Self::Project { v, frame, plane } => Self::Project {
                    v: v.clone_ref(py),
                    frame: frame.clone_ref(py),
                    plane: *plane,
                },
            }
        })
    }
}
/// StateSpec allows defining a state from the target to the observer
#[derive(Clone)]
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

    /// Convert the S-Expression to a StateSpec
    /// :type expr: str
    /// :rtype: StateSpec
    #[classmethod]
    fn from_s_expr(_cls: Bound<'_, PyType>, expr: &str) -> Result<Self, PyErr> {
        let spec = StateSpec::from_s_expr(expr).map_err(|e| PyException::new_err(e.to_string()))?;

        spec.try_into()
    }

    /// Converts this StateSpec to its S-Expression
    /// :rtype: str
    fn to_s_expr(&self) -> Result<String, PyErr> {
        let spec = StateSpec::from(self.clone());

        spec.to_s_expr()
            .map_err(|e| PyException::new_err(e.to_string()))
    }
    /// Evaluate the orbital element enum variant for the provided orbit
    #[pyo3(name = "evaluate", signature=(epoch, almanac))]
    fn py_evaluate(&self, epoch: Epoch, almanac: &Almanac) -> Result<Orbit, PyErr> {
        let spec = StateSpec::from(self.clone());
        spec.evaluate(epoch, almanac)
            .map_err(|e| PyException::new_err(e.to_string()))
    }
    fn __eq__(&self, other: &Self) -> bool {
        let me = StateSpec::from(self.clone());
        let other = StateSpec::from(other.clone());
        me == other
    }
    fn __ne__(&self, other: &Self) -> bool {
        let me = StateSpec::from(self.clone());
        let other = StateSpec::from(other.clone());
        me != other
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

impl Clone for PyFrameSpec {
    fn clone(&self) -> Self {
        Python::attach(|py| -> PyFrameSpec {
            match self {
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

// TODO: Implement clones manually for eveyrthing
impl Clone for PyOrthogonalFrame {
    fn clone(&self) -> Self {
        Python::attach(|py| -> PyOrthogonalFrame {
            match self {
                Self::XY { x, y } => Self::XY {
                    x: x.clone_ref(py),
                    y: y.clone_ref(py),
                },
                Self::XZ { x, z } => Self::XZ {
                    x: x.clone_ref(py),
                    z: z.clone_ref(py),
                },
                Self::YZ { y, z } => Self::YZ {
                    z: z.clone_ref(py),
                    y: y.clone_ref(py),
                },
            }
        })
    }
}
// *** Implement the From<RustType> for PythonType to convert the LISP representation *** //

impl TryFrom<ScalarExpr> for PyScalarExpr {
    type Error = PyErr;

    fn try_from(value: ScalarExpr) -> Result<Self, Self::Error> {
        Python::attach(|py| -> Result<Self, PyErr> {
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
                ScalarExpr::GravParam { celestial_object } => {
                    Ok(Self::GravParam { celestial_object })
                }
                ScalarExpr::Norm(v) => Ok(Self::Norm(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?)),
                ScalarExpr::NormSquared(v) => Ok(Self::NormSquared(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?)),
                ScalarExpr::VectorX(v) => Ok(Self::VectorX(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?)),
                ScalarExpr::VectorY(v) => Ok(Self::VectorY(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?)),
                ScalarExpr::VectorZ(v) => Ok(Self::VectorZ(Py::new(
                    py,
                    <VectorExpr as TryInto<PyVectorExpr>>::try_into(v)?,
                )?)),
                ScalarExpr::DotProduct { a, b } => Ok(Self::DotProduct {
                    a: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(a)?)?,
                    b: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(b)?)?,
                }),
                ScalarExpr::AngleBetween { a, b } => Ok(Self::AngleBetween {
                    a: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(a)?)?,
                    b: Py::new(py, <VectorExpr as TryInto<PyVectorExpr>>::try_into(b)?)?,
                }),
                ScalarExpr::Negate(v) => Ok(Self::Negate(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Invert(v) => Ok(Self::Invert(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Cos(v) => Ok(Self::Cos(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Sin(v) => Ok(Self::Sin(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Tan(v) => Ok(Self::Tan(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Acos(v) => Ok(Self::Acos(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Asin(v) => Ok(Self::Asin(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Sqrt(v) => Ok(Self::Sqrt(Py::new(
                    py,
                    <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?,
                )?)),
                ScalarExpr::Powi { scalar, n } => Ok(Self::Powi {
                    scalar: Py::new(
                        py,
                        <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*scalar)?,
                    )?,
                    n,
                }),
                ScalarExpr::Powf { scalar, n } => Ok(Self::Powf {
                    scalar: Py::new(
                        py,
                        <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*scalar)?,
                    )?,
                    n,
                }),
                ScalarExpr::Add { a, b } => Ok(Self::Add {
                    a: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*a)?)?,
                    b: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*b)?)?,
                }),
                ScalarExpr::Mul { a, b } => Ok(Self::Mul {
                    a: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*a)?)?,
                    b: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*b)?)?,
                }),
                ScalarExpr::Atan2 { y, x } => Ok(Self::Atan2 {
                    y: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*y)?)?,
                    x: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*x)?)?,
                }),
                ScalarExpr::Modulo { v, m } => Ok(Self::Modulo {
                    v: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*v)?)?,
                    m: Py::new(py, <ScalarExpr as TryInto<PyScalarExpr>>::try_into(*m)?)?,
                }),
            }
        })
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

// *** Converse *** //

impl From<PyScalarExpr> for ScalarExpr {
    fn from(value: PyScalarExpr) -> Self {
        Python::attach(|py| match value {
            // --- Direct Conversions (unaffected by the change but now run inside the GIL) ---
            PyScalarExpr::Constant(c) => ScalarExpr::Constant(c),
            PyScalarExpr::MeanEquatorialRadius { celestial_object } => {
                ScalarExpr::MeanEquatorialRadius { celestial_object }
            }
            PyScalarExpr::SemiMajorEquatorialRadius { celestial_object } => {
                ScalarExpr::SemiMajorEquatorialRadius { celestial_object }
            }
            PyScalarExpr::SemiMinorEquatorialRadius { celestial_object } => {
                ScalarExpr::SemiMinorEquatorialRadius { celestial_object }
            }
            PyScalarExpr::PolarRadius { celestial_object } => {
                ScalarExpr::PolarRadius { celestial_object }
            }
            PyScalarExpr::Flattening { celestial_object } => {
                ScalarExpr::Flattening { celestial_object }
            }
            PyScalarExpr::GravParam { celestial_object } => {
                ScalarExpr::GravParam { celestial_object }
            }
            PyScalarExpr::Element(e) => ScalarExpr::Element(e),
            PyScalarExpr::SolarEclipsePercentage { eclipsing_frame } => {
                ScalarExpr::SolarEclipsePercentage { eclipsing_frame }
            }
            PyScalarExpr::OccultationPercentage {
                back_frame,
                front_frame,
            } => ScalarExpr::OccultationPercentage {
                back_frame,
                front_frame,
            },
            PyScalarExpr::BetaAngle() => ScalarExpr::BetaAngle,
            PyScalarExpr::LocalSolarTime() => ScalarExpr::LocalSolarTime,
            PyScalarExpr::LocalTimeAscNode() => ScalarExpr::LocalTimeAscNode,
            PyScalarExpr::LocalTimeDescNode() => ScalarExpr::LocalTimeDescNode,
            PyScalarExpr::SunAngle { observer_id } => ScalarExpr::SunAngle { observer_id },
            PyScalarExpr::AzimuthFromLocation {
                location_id,
                obstructing_body,
            } => ScalarExpr::AzimuthFromLocation {
                location_id,
                obstructing_body,
            },
            PyScalarExpr::ElevationFromLocation {
                location_id,
                obstructing_body,
            } => ScalarExpr::ElevationFromLocation {
                location_id,
                obstructing_body,
            },
            PyScalarExpr::RangeFromLocation {
                location_id,
                obstructing_body,
            } => ScalarExpr::RangeFromLocation {
                location_id,
                obstructing_body,
            },
            PyScalarExpr::RangeRateFromLocation {
                location_id,
                obstructing_body,
            } => ScalarExpr::RangeRateFromLocation {
                location_id,
                obstructing_body,
            },

            // --- Recursive Conversions (now using the acquired `py` token) ---
            PyScalarExpr::Add { a, b } => ScalarExpr::Add {
                a: Box::new(a.borrow(py).clone().into()),
                b: Box::new(b.borrow(py).clone().into()),
            },
            PyScalarExpr::Mul { a, b } => ScalarExpr::Mul {
                a: Box::new(a.borrow(py).clone().into()),
                b: Box::new(b.borrow(py).clone().into()),
            },
            PyScalarExpr::Negate(s) => ScalarExpr::Negate(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Invert(s) => ScalarExpr::Invert(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Sqrt(s) => ScalarExpr::Sqrt(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Powi { scalar, n } => ScalarExpr::Powi {
                scalar: Box::new(scalar.borrow(py).clone().into()),
                n,
            },
            PyScalarExpr::Powf { scalar, n } => ScalarExpr::Powf {
                scalar: Box::new(scalar.borrow(py).clone().into()),
                n,
            },
            PyScalarExpr::Cos(s) => ScalarExpr::Cos(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Sin(s) => ScalarExpr::Sin(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Tan(s) => ScalarExpr::Tan(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Acos(s) => ScalarExpr::Acos(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Asin(s) => ScalarExpr::Asin(Box::new(s.borrow(py).clone().into())),
            PyScalarExpr::Atan2 { y, x } => ScalarExpr::Atan2 {
                y: Box::new(y.borrow(py).clone().into()),
                x: Box::new(x.borrow(py).clone().into()),
            },
            PyScalarExpr::Modulo { v, m } => ScalarExpr::Modulo {
                v: Box::new(v.borrow(py).clone().into()),
                m: Box::new(m.borrow(py).clone().into()),
            },
            PyScalarExpr::Norm(v) => ScalarExpr::Norm(v.borrow(py).clone().into()),
            PyScalarExpr::NormSquared(v) => ScalarExpr::NormSquared(v.borrow(py).clone().into()),
            PyScalarExpr::DotProduct { a, b } => ScalarExpr::DotProduct {
                a: a.borrow(py).clone().into(),
                b: b.borrow(py).clone().into(),
            },
            PyScalarExpr::AngleBetween { a, b } => ScalarExpr::AngleBetween {
                a: a.borrow(py).clone().into(),
                b: b.borrow(py).clone().into(),
            },
            PyScalarExpr::VectorX(v) => ScalarExpr::VectorX(v.borrow(py).clone().into()),
            PyScalarExpr::VectorY(v) => ScalarExpr::VectorY(v.borrow(py).clone().into()),
            PyScalarExpr::VectorZ(v) => ScalarExpr::VectorZ(v.borrow(py).clone().into()),
        })
    }
}
impl From<PyVectorExpr> for VectorExpr {
    fn from(value: PyVectorExpr) -> Self {
        Python::attach(|py| match value {
            PyVectorExpr::Fixed { x, y, z } => VectorExpr::Fixed { x, y, z },
            PyVectorExpr::Radius(spec) => VectorExpr::Radius(spec.borrow(py).clone().into()),
            PyVectorExpr::Velocity(spec) => VectorExpr::Velocity(spec.borrow(py).clone().into()),
            PyVectorExpr::OrbitalMomentum(spec) => {
                VectorExpr::OrbitalMomentum(spec.borrow(py).clone().into())
            }
            PyVectorExpr::EccentricityVector(spec) => {
                VectorExpr::EccentricityVector(spec.borrow(py).clone().into())
            }
            PyVectorExpr::CrossProduct { a, b } => VectorExpr::CrossProduct {
                a: Box::new(a.borrow(py).clone().into()),
                b: Box::new(b.borrow(py).clone().into()),
            },
            PyVectorExpr::VecProjection { a, b } => VectorExpr::VecProjection {
                a: Box::new(a.borrow(py).clone().into()),
                b: Box::new(b.borrow(py).clone().into()),
            },
            PyVectorExpr::Unit(v) => VectorExpr::Unit(Box::new(v.borrow(py).clone().into())),
            PyVectorExpr::Negate(v) => VectorExpr::Negate(Box::new(v.borrow(py).clone().into())),
            PyVectorExpr::Project { v, frame, plane } => VectorExpr::Project {
                v: Box::new(v.borrow(py).clone().into()),
                frame: Box::new(frame.borrow(py).clone().into()),
                plane,
            },
        })
    }
}

impl From<PyOrthogonalFrame> for OrthogonalFrame {
    fn from(value: PyOrthogonalFrame) -> Self {
        Python::attach(|py| match value {
            PyOrthogonalFrame::XY { x, y } => OrthogonalFrame::XY {
                x: x.borrow(py).clone().into(),
                y: y.borrow(py).clone().into(),
            },
            PyOrthogonalFrame::XZ { x, z } => OrthogonalFrame::XZ {
                x: x.borrow(py).clone().into(),
                z: z.borrow(py).clone().into(),
            },
            PyOrthogonalFrame::YZ { y, z } => OrthogonalFrame::YZ {
                y: y.borrow(py).clone().into(),
                z: z.borrow(py).clone().into(),
            },
        })
    }
}

impl From<PyStateSpec> for StateSpec {
    fn from(value: PyStateSpec) -> Self {
        StateSpec {
            target_frame: value.target_frame.into(),
            observer_frame: value.observer_frame.into(),
            ab_corr: value.ab_corr,
        }
    }
}

impl From<PyFrameSpec> for FrameSpec {
    fn from(value: PyFrameSpec) -> Self {
        match value {
            PyFrameSpec::Loaded(frame) => Self::Loaded(frame),
            PyFrameSpec::Manual { name, defn } => Python::attach(|py| {
                let py_ortho: PyOrthogonalFrame = defn.borrow(py).clone();

                Self::Manual {
                    name,
                    defn: Box::new(py_ortho.into()),
                }
            }),
        }
    }
}
