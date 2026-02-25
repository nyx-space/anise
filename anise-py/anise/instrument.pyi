from __future__ import annotations
from anise import astro
from anise import rotation
import numpy
import typing

@typing.final
class Ellipsoid:
    """Only the tri-axial Ellipsoid shape model is currently supported by ANISE.
This is directly inspired from SPICE PCK.
> For each body, three radii are listed: The first number is
> the largest equatorial radius (the length of the semi-axis
> containing the prime meridian), the second number is the smaller
> equatorial radius, and the third is the polar radius.

Example: Radii of the Earth.

BODY399_RADII     = ( 6378.1366   6378.1366   6356.7519 )"""
    polar_radius_km: float
    semi_major_equatorial_radius_km: float
    semi_minor_equatorial_radius_km: float

    def __init__(self, *args: typing.Any, **kwargs: typing.Any) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""

    def __new__(cls, semi_major_equatorial_radius_km: float, polar_radius_km: typing.Optional[float]=None, semi_minor_equatorial_radius_km: typing.Optional[float]=None) -> Ellipsoid:
        """Only the tri-axial Ellipsoid shape model is currently supported by ANISE.
This is directly inspired from SPICE PCK.
> For each body, three radii are listed: The first number is
> the largest equatorial radius (the length of the semi-axis
> containing the prime meridian), the second number is the smaller
> equatorial radius, and the third is the polar radius.

Example: Radii of the Earth.

BODY399_RADII     = ( 6378.1366   6378.1366   6356.7519 )"""

    def flattening(self) -> float:
        """Returns the flattening ratio, computed from the mean equatorial radius and the polar radius"""

    def is_sphere(self) -> bool:
        """Returns true if the polar radius is equal to the semi minor radius."""

    def is_spheroid(self) -> bool:
        """Returns true if the semi major and minor radii are equal"""

    def mean_equatorial_radius_km(self) -> float:
        """Returns the mean equatorial radius in kilometers"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __getnewargs__(self) -> typing.Tuple:
        """Allows for pickling the object"""

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
class FovShape:

    def __init__(self, *args: typing.Any, **kwargs: typing.Any) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""

    def __new__(cls):...
    Conical: type = ...
    Rectangular: type = ...

@typing.final
class Instrument:
    """Instrument is defined by a mounting Euler Parameter, a mounting translation ("level arm"), and a field of view of the instrument.
Notations: frame N is inertial; frame B is body; frame I is instrument."""
    fov: FovShape
    offset_i: numpy.ndarray
    q_to_i: rotation.Quaternion

    def __init__(self, *args: typing.Any, **kwargs: typing.Any) -> None:
        """Initialize self.  See help(type(self)) for accurate signature."""

    def __new__(cls, q_to_i: typing.Any, offset: typing.Any, fov: typing.Any) -> Instrument:
        """Instrument is defined by a mounting Euler Parameter, a mounting translation ("level arm"), and a field of view of the instrument.
Notations: frame N is inertial; frame B is body; frame I is instrument."""

    def footprint(self, sc_q_n_to_b: rotation.Quaternion, sc_state_target: astro.Orbit, q_n_to_target: rotation.Quaternion, resolution: int) -> list[astro.Orbit]:
        """Computes the footprint (swath) of the instrument on a target body.

This function projects the edges of the Field of View onto the provided target ellipsoid.

# Arguments
* `sc_q_n_to_b` - The orientation of the spacecraft body relative to Inertial.
* `sc_state_target` - The inertial state (position/velocity) of the spacecraft.
* `q_n_to_target` - The orientation of the target body frame relative to Inertial.
* `resolution` - The number of points to generate along the FOV boundary.

# Returns
A vector of `Orbit` objects, each representing a point on the surface of the target
expressed in the `target_frame` (Fixed)."""

    def fov_margin_deg(self, sc_q_to_b: rotation.Quaternion, sc_state: astro.Orbit, target_state: astro.Orbit) -> float:
        """Calculates the angular margin to the FOV boundary in degrees.

# Arguments
* sc_q_to_b: rotation from the sc_state frame to the body frame in which is expressed the instrument rotation.
* sc_state: state of the spacecraft, typically in an inertial frame
* target_state: state of the target object in the same frame as the sc_state, e.g. IAU Moon if sc_state is in IAU Moon

This is a continuous function suitable for event detection (root finding).
* `> 0.0`: Target is INSIDE.
* `< 0.0`: Target is OUTSIDE.
* `= 0.0`: Target is ON THE BOUNDARY.

NOTE: This call will return an error if the reference frames are not adequate.
Example:
- If the mounting rotation "from" frame does not match in sc_attitude_to_body "to" frame IDs
- If the target state frame ID is not identical to the instrument's inertial state given the sc_attitude Euler Parameter."""

    def is_target_in_fov(self, sc_attitude_inertial_to_body: rotation.Quaternion, sc_state: astro.Orbit, target_state: astro.Orbit) -> bool:
        """Checks if a target is visible within the Field of View."""

    def transform_state(self, q_sc_to_b: rotation.Quaternion, sc_state: astro.Orbit) -> tuple[rotation.Quaternion, astro.Orbit]:
        """Computes the state (orientation + Cartesian state) of the instrument
at a specific instant, given the spacecraft's state.

NOTE: This call will return an error if the reference frames are not adequate.
Example:
- If the mounting quaterion (q_to_i) frame does not match in sc_attitude_to_body "to" frame IDs"""