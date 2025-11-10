import typing

from anise import Aberration, Almanac
from anise.astro import Frame, Orbit
from anise.time import Duration, Epoch

@typing.final
class Event:
    """Defines a state parameter event finder from the desired value of the scalar expression to compute, precision on timing and value, and the aberration."""
    ab_corr: Aberration
    desired_value: float
    epoch_precision: Duration
    scalar: ScalarExpr
    value_precision: float

    def __init__(self, scalar: ScalarExpr, desired_value: float, epoch_precision: Duration, value_precision: float, ab_corr: Aberration=None) -> None:
        """Defines a state parameter event finder from the desired value of the scalar expression to compute, precision on timing and value, and the aberration."""

    @staticmethod
    def above_horizon_from_location_id(location_id: int, obstructing_body: Frame=None) -> Event:
        """Report events where the object is above the horizon when seen from the provided location ID."""

    @staticmethod
    def apoapsis() -> Event:
        """Apoapsis event finder, with an epoch precision of 0.1 seconds"""

    @staticmethod
    def eclipse(eclipsing_frame: Frame) -> Event:
        """Total eclipse event finder: returns events where the eclipsing percentage is greater than 98.9%."""

    def eval(self, orbit: Orbit, almanac: Almanac) -> float:
        """Compute the event finding function of this event provided an Orbit and Almanac."""

    def eval_string(self, orbit: Orbit, almanac: Almanac) -> str:
        """Pretty print the evaluation of this event for the provided Orbit and Almanac"""

    @staticmethod
    def from_s_expr(expr: str) -> Event:
        """Convert the S-Expression to a Event"""

    @staticmethod
    def periapsis() -> Event:
        """Periapsis event finder, with an epoch precision of 0.1 seconds"""

    def to_s_expr(self) -> str:
        """Converts this Event to its S-Expression"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class EventArc:
    fall: EventDetails
    rise: EventDetails

    def __init__(self):...

    def duration(self):...

    def end_epoch(self):...

    def start_epoch(self):...

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class EventDetails:
    """Represents the details of an event occurring along a trajectory.

`EventDetails` encapsulates the state at which a particular event occurs in a trajectory, along with additional information about the nature of the event. This struct is particularly useful for understanding the dynamics of the event, such as whether it represents a rising or falling edge, or if the edge is unclear.

# Generics
S: Interpolatable - A type that represents the state of the trajectory. This type must implement the `Interpolatable` trait, ensuring that it can be interpolated and manipulated according to the trajectory's requirements."""
    edge: EventEdge
    next_value: float
    orbit: Orbit
    pm_duration: Duration
    prev_value: float
    repr: str
    value: float

    def __init__(self) -> None:
        """Represents the details of an event occurring along a trajectory.

`EventDetails` encapsulates the state at which a particular event occurs in a trajectory, along with additional information about the nature of the event. This struct is particularly useful for understanding the dynamics of the event, such as whether it represents a rising or falling edge, or if the edge is unclear.

# Generics
S: Interpolatable - A type that represents the state of the trajectory. This type must implement the `Interpolatable` trait, ensuring that it can be interpolated and manipulated according to the trajectory's requirements."""

    def describe(self) -> str:...

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class EventEdge:
    """Enumerates the possible edges of an event in a trajectory.

`EventEdge` is used to describe the nature of a trajectory event, particularly in terms of its temporal dynamics relative to a specified condition or threshold. This enum helps in distinguishing whether the event is occurring at a rising edge, a falling edge, or if the edge is unclear due to insufficient data or ambiguous conditions."""

    def __init__(self) -> None:
        """Enumerates the possible edges of an event in a trajectory.

`EventEdge` is used to describe the nature of a trajectory event, particularly in terms of its temporal dynamics relative to a specified condition or threshold. This enum helps in distinguishing whether the event is occurring at a rising edge, a falling edge, or if the edge is unclear due to insufficient data or ambiguous conditions."""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __int__(self) -> None:
        """int(self)"""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __repr__(self) -> str:
        """Return repr(self)."""
    Falling: EventEdge = ...
    LocalMax: EventEdge = ...
    LocalMin: EventEdge = ...
    Rising: EventEdge = ...
    Unclear: EventEdge = ...

@typing.final
class FrameSpec:
    """FrameSpec allows defining a frame that can be computed from another set of loaded frames, which include a center."""

    def __init__(self) -> None:
        """FrameSpec allows defining a frame that can be computed from another set of loaded frames, which include a center."""
    Loaded: type = ...
    Manual: type = ...

@typing.final
class OrbitalElement:
    """Orbital element defines all of the supported orbital elements in ANISE, which are all built from a State."""

    def __init__(self) -> None:
        """Orbital element defines all of the supported orbital elements in ANISE, which are all built from a State."""

    def evaluate(self, orbit: Orbit) -> float:
        """Evaluate the orbital element enum variant for the provided orbit"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __int__(self) -> None:
        """int(self)"""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __repr__(self) -> str:
        """Return repr(self)."""
    AoL: OrbitalElement = ...
    AoP: OrbitalElement = ...
    ApoapsisAltitude: OrbitalElement = ...
    ApoapsisRadius: OrbitalElement = ...
    C3: OrbitalElement = ...
    Declination: OrbitalElement = ...
    EccentricAnomaly: OrbitalElement = ...
    Eccentricity: OrbitalElement = ...
    Energy: OrbitalElement = ...
    FlightPathAngle: OrbitalElement = ...
    HX: OrbitalElement = ...
    HY: OrbitalElement = ...
    HZ: OrbitalElement = ...
    Height: OrbitalElement = ...
    Hmag: OrbitalElement = ...
    HyperbolicAnomaly: OrbitalElement = ...
    Inclination: OrbitalElement = ...
    Latitude: OrbitalElement = ...
    Longitude: OrbitalElement = ...
    MeanAnomaly: OrbitalElement = ...
    PeriapsisAltitude: OrbitalElement = ...
    PeriapsisRadius: OrbitalElement = ...
    Period: OrbitalElement = ...
    RAAN: OrbitalElement = ...
    RightAscension: OrbitalElement = ...
    Rmag: OrbitalElement = ...
    SemiMajorAxis: OrbitalElement = ...
    SemiMinorAxis: OrbitalElement = ...
    SemiParameter: OrbitalElement = ...
    TrueAnomaly: OrbitalElement = ...
    TrueLongitude: OrbitalElement = ...
    VX: OrbitalElement = ...
    VY: OrbitalElement = ...
    VZ: OrbitalElement = ...
    VelocityDeclination: OrbitalElement = ...
    Vmag: OrbitalElement = ...
    X: OrbitalElement = ...
    Y: OrbitalElement = ...
    Z: OrbitalElement = ...

@typing.final
class OrthogonalFrame:

    def __init__(self):...
    XY: type = ...
    XZ: type = ...
    YZ: type = ...

@typing.final
class Plane:
    """Plane selector, sets the missing component to zero.
For example, Plane::YZ will multiply the DCM by [[1, 0. 0], [0, 1, 0], [0, 0, 0]]"""

    def __init__(self) -> None:
        """Plane selector, sets the missing component to zero.
For example, Plane::YZ will multiply the DCM by [[1, 0. 0], [0, 1, 0], [0, 0, 0]]"""

    def __int__(self) -> None:
        """int(self)"""

    def __repr__(self) -> str:
        """Return repr(self)."""
    XY: Plane = ...
    XZ: Plane = ...
    YZ: Plane = ...

@typing.final
class ReportScalars:
    """A basic report builder that can be serialized seperately from the execution.
The scalars must be a tuple of (ScalarExpr, String) where the String is the alias (optional)."""

    def __init__(self, scalars: list, state_spec: StateSpec) -> ReportScalars:
        """A basic report builder that can be serialized seperately from the execution.
The scalars must be a tuple of (ScalarExpr, String) where the String is the alias (optional)."""

    @staticmethod
    def from_s_expr(expr: str) -> ReportScalars:
        """Convert the S-Expression to a report builder"""

    def to_s_expr(self) -> str:
        """Converts this report builder to its S-Expression"""

@typing.final
class ScalarExpr:
    """ScalarExpr defines a scalar computation from a (set of) vector expression(s)."""

    def __init__(self) -> None:
        """ScalarExpr defines a scalar computation from a (set of) vector expression(s)."""

    def evaluate(self, orbit: Orbit, almanac: Almanac, ab_corr: Aberration=None) -> float:
        """Compute this ScalarExpr for the provided Orbit"""

    @staticmethod
    def from_s_expr(expr: str) -> ScalarExpr:
        """Convert the S-Expression to a ScalarExpr"""

    def to_s_expr(self) -> str:
        """Converts this ScalarExpr to its S-Expression"""
    Acos: type = ...
    Add: type = ...
    AngleBetween: type = ...
    Asin: type = ...
    Atan2: type = ...
    AzimuthFromLocation: type = ...
    BetaAngle: type = ...
    Constant: type = ...
    Cos: type = ...
    DotProduct: type = ...
    Element: type = ...
    ElevationFromLocation: type = ...
    Flattening: type = ...
    GravParam: type = ...
    Invert: type = ...
    LocalSolarTime: type = ...
    LocalTimeAscNode: type = ...
    LocalTimeDescNode: type = ...
    MeanEquatorialRadius: type = ...
    Modulo: type = ...
    Mul: type = ...
    Negate: type = ...
    Norm: type = ...
    NormSquared: type = ...
    OccultationPercentage: type = ...
    PolarRadius: type = ...
    Powf: type = ...
    Powi: type = ...
    RangeFromLocation: type = ...
    RangeRateFromLocation: type = ...
    SemiMajorEquatorialRadius: type = ...
    SemiMinorEquatorialRadius: type = ...
    Sin: type = ...
    SolarEclipsePercentage: type = ...
    Sqrt: type = ...
    SunAngle: type = ...
    Tan: type = ...
    VectorX: type = ...
    VectorY: type = ...
    VectorZ: type = ...

@typing.final
class StateSpec:
    """StateSpec allows defining a state from the target to the observer"""
    ab_corr: Aberration
    observer_frame: FrameSpec
    target_frame: FrameSpec

    def __init__(self, target_frame: FrameSpec, observer_frame: FrameSpec, ab_corr: Aberration=None) -> None:
        """StateSpec allows defining a state from the target to the observer"""

    def evaluate(self, epoch: Epoch, almanac: Almanac) -> Orbit:
        """Evaluate the orbital element enum variant for the provided orbit"""

    @staticmethod
    def from_s_expr(expr: str) -> StateSpec:
        """Convert the S-Expression to a StateSpec"""

    def to_s_expr(self) -> str:
        """Converts this StateSpec to its S-Expression"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

@typing.final
class VectorExpr:

    def __init__(self):...
    CrossProduct: type = ...
    EccentricityVector: type = ...
    Fixed: type = ...
    Negate: type = ...
    OrbitalMomentum: type = ...
    Project: type = ...
    Radius: type = ...
    Unit: type = ...
    VecProjection: type = ...
    Velocity: type = ...
