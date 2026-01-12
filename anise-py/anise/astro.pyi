import np
import numpy
import typing

@typing.final
class AzElRange:
    """A structure that stores the result of Azimuth, Elevation, Range, Range rate calculation."""
    azimuth_deg: float
    elevation_deg: float
    epoch: Epoch
    light_time: Duration
    mask_deg: float
    obstructed_by: Frame
    range_km: float
    range_rate_km_s: float

    def __init__(self, epoch: Epoch, azimuth_deg: float, elevation_deg: float, range_km: float, range_rate_km_s: float, obstructed_by: Frame=None, mask_deg: float=None) -> AzElRange:
        """A structure that stores the result of Azimuth, Elevation, Range, Range rate calculation."""

    def elevation_above_mask_deg(self) -> float:
        """Returns the elevation above the terrain mask for this azimuth, in degrees.
If the terrain mask was zero at this azimuth, then the elevation above mask is equal to the elevation_deg field."""

    def is_obstructed(self) -> bool:
        """Returns whether there is an obstruction."""

    def is_valid(self) -> bool:
        """Returns false if the range is less than one millimeter, or any of the angles are NaN."""

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
class Covariance:
    matrix: numpy.array

    def __init__(self):...

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class DataType:

    def __init__(self):...

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
    Type10SpaceCommandTLE: DataType = ...
    Type12HermiteEqualStep: DataType = ...
    Type13HermiteUnequalStep: DataType = ...
    Type14ChebyshevUnequalStep: DataType = ...
    Type15PrecessingConics: DataType = ...
    Type17Equinoctial: DataType = ...
    Type18ESOCHermiteLagrange: DataType = ...
    Type19ESOCPiecewise: DataType = ...
    Type1ModifiedDifferenceArray: DataType = ...
    Type20ChebyshevDerivative: DataType = ...
    Type21ExtendedModifiedDifferenceArray: DataType = ...
    Type2ChebyshevTriplet: DataType = ...
    Type3ChebyshevSextuplet: DataType = ...
    Type5DiscreteStates: DataType = ...
    Type8LagrangeEqualStep: DataType = ...
    Type9LagrangeUnequalStep: DataType = ...

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

    def __init__(self, semi_major_equatorial_radius_km: float, polar_radius_km: float=None, semi_minor_equatorial_radius_km: float=None) -> Ellipsoid:
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
class Ephemeris:
    """Initializes a new Ephemeris from the list of Orbit instances and a given object ID.

In Python if you need to build an ephemeris with covariance, initialize with an empty list of
orbit instances and then insert each EphemEntry with covariance."""
    interpolation: str
    object_id: str

    def __init__(self, orbit_list: list, object_id: str) -> None:
        """Initializes a new Ephemeris from the list of Orbit instances and a given object ID.

In Python if you need to build an ephemeris with covariance, initialize with an empty list of
orbit instances and then insert each EphemEntry with covariance."""

    def at(self, epoch: Epoch, almanac: Almanac) -> EphemerisRecord:
        """Interpolates the ephemeris state and covariance at the provided epoch.

# Orbit Interpolation
The orbital state is interpolated using high-fidelity numeric methods consistent
with SPICE standards:
* **Type 9 (Lagrange):** Uses an Nth-order Lagrange polynomial interpolation on
unequal time steps. It interpolates each of the 6 state components (position
and velocity) independently.
* **Type 13 (Hermite):** Uses an Nth-order Hermite interpolation. This method
explicitly uses the velocity data (derivatives) to constrain the interpolation
of the position, ensuring that the resulting position curve is smooth and
dynamically consistent with the velocity.

# Covariance Interpolation (Log-Euclidean)
If covariance data is available, this method performs **Log-Euclidean Riemannian
Interpolation**. Unlike standard linear element-wise interpolation, this approach
respects the geometric manifold of Symmetric Positive Definite (SPD) matrices.

This guarantees that:
1. **Positive Definiteness:** The interpolated covariance matrix is always mathematically
valid (all eigenvalues are strictly positive), preventing numerical crashes in downstream filters.
2. **Volume Preservation:** It prevents the artificial "swelling" (determinant increase)
of uncertainty that occurs when linearly interpolating between two valid matrices.
The interpolation follows the "geodesic" (shortest path) on the curved surface of
covariance matrices."""

    def covar_at(self, epoch: Epoch, local_frame: LocalFrame, almanac: Almanac) -> Covariance:
        """Interpolate the ephemeris at the provided epoch, returning only the covariance."""

    def domain(self) -> tuple:
        """Returns the time domain of this ephemeris."""

    @staticmethod
    def from_ccsds_oem_file(path: str) -> Ephemeris:
        """Initializes a new Ephemeris from a file path to CCSDS OEM file."""

    @staticmethod
    def from_stk_e_file(path: str) -> Ephemeris:
        """Initializes a new Ephemeris from a file path to Ansys STK .e file."""

    def includes_covariance(self) -> bool:
        """Returns whether all of the data in this ephemeris includes the covariance."""

    def insert(self, entry: EphemerisRecord) -> None:
        """Inserts a new ephemeris entry to this ephemeris (it is automatically sorted chronologically)."""

    def insert_orbit(self, orbit: Orbit) -> None:
        """Inserts a new orbit (without covariance) to this ephemeris (it is automatically sorted chronologically)."""

    def nearest_after(self, epoch: Epoch, almanac: Almanac) -> EphemerisRecord:
        """Returns the nearest entry after the provided time"""

    def nearest_before(self, epoch: Epoch, almanac: Almanac) -> EphemerisRecord:
        """Returns the nearest entry before the provided time"""

    def nearest_covar_after(self, epoch: Epoch, almanac: Almanac) -> tuple:
        """Returns the nearest covariance after the provided epoch as a tuple (Epoch, Covariance)"""

    def nearest_covar_before(self, epoch: Epoch, almanac: Almanac) -> tuple:
        """Returns the nearest covariance before the provided epoch as a tuple (Epoch, Covariance)"""

    def nearest_orbit_after(self, epoch: Epoch, almanac: Almanac) -> Orbit:
        """Returns the nearest orbit after the provided time"""

    def nearest_orbit_before(self, epoch: Epoch, almanac: Almanac) -> Orbit:
        """Returns the nearest orbit before the provided time"""

    def orbit_at(self, epoch: Epoch, almanac: Almanac) -> Orbit:
        """Interpolate the ephemeris at the provided epoch, returning only the orbit."""

    def resample(self, ts: TimeSeries, almanac: Almanac) -> Ephemeris:
        """Resample this ephemeris, with covariance, at the provided time series"""

    def to_ccsds_oem_file(self, path: str, originator: str=None, object_name: str=None) -> None:
        """Exports this Ephemeris to CCSDS OEM at the provided path, optionally specifying an originator and/or an object name"""

    def write_spice_bsp(self, naif_id: int, output_fname: str, data_type: DataType) -> None:
        """Converts this ephemeris to SPICE BSP/SPK file in the provided data type, saved to the provided output_fname."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class EphemerisRecord:
    covar: Covariance
    orbit: Orbit

    def __init__(self):...

    def covar_in_frame(self, local_frame: LocalFrame) -> Covariance:
        """Returns the covariance in the desired orbit local frame, or None if this record does not define a covariance."""

    def sigma_for(self, oe: OrbitalElement) -> float:
        """Returns the 1-sigma uncertainty (Standard Deviation) for a given orbital element.

The result is in the unit of the parameter (e.g., km for SMA, degrees for angles).

This method uses the [OrbitGrad] structure (Hyperdual numbers) to compute the
Jacobian of the element with respect to the inertial Cartesian state, and then
rotates the covariance into that hyperdual dual space: J * P * J^T."""

@typing.final
class Frame:
    """A Frame uniquely defined by its ephemeris center and orientation. Refer to FrameDetail for frames combined with parameters."""
    ephemeris_id: int
    orientation_id: int
    shape: Ellipsoid

    def __init__(self, ephemeris_id: int, orientation_id: int, mu_km3_s2: float=None, shape: Ellipsoid=None) -> Frame:
        """A Frame uniquely defined by its ephemeris center and orientation. Refer to FrameDetail for frames combined with parameters."""

    def ephem_origin_id_match(self, other_id: int) -> bool:
        """Returns true if the ephemeris origin is equal to the provided ID"""

    def ephem_origin_match(self, other: Frame) -> bool:
        """Returns true if the ephemeris origin is equal to the provided frame"""

    def flattening(self) -> float:
        """Returns the flattening ratio (unitless)"""

    def is_celestial(self) -> bool:
        """Returns whether this is a celestial frame"""

    def is_geodetic(self) -> bool:
        """Returns whether this is a geodetic frame"""

    def mean_equatorial_radius_km(self) -> float:
        """Returns the mean equatorial radius in km, if defined"""

    def mu_km3_s2(self) -> float:
        """Returns the gravitational parameters of this frame, if defined"""

    def orient_origin_id_match(self, other_id: int) -> bool:
        """Returns true if the orientation origin is equal to the provided ID"""

    def orient_origin_match(self, other: Frame) -> bool:
        """Returns true if the orientation origin is equal to the provided frame"""

    def polar_radius_km(self) -> float:
        """Returns the polar radius in km, if defined"""

    def semi_major_radius_km(self) -> float:
        """Returns the semi major radius of the tri-axial ellipoid shape of this frame, if defined"""

    def strip(self) -> None:
        """Removes the graviational parameter and the shape information from this frame.
Use this to prevent astrodynamical computations."""

    def with_ephem(self, new_ephem_id: int) -> Frame:
        """Returns a copy of this Frame whose ephemeris ID is set to the provided ID"""

    def with_mu_km3_s2(self, mu_km3_s2: float) -> Frame:
        """Returns a copy of this frame with the graviational parameter set to the new value."""

    def with_orient(self, new_orient_id: int) -> Frame:
        """Returns a copy of this Frame whose orientation ID is set to the provided ID"""

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
class FrameUid:
    """A unique frame reference that only contains enough information to build the actual Frame object.
It cannot be used for any computations, is it be used in any structure apart from error structures."""

    def __init__(self, ephemeris_id: int, orientation_id: int) -> None:
        """A unique frame reference that only contains enough information to build the actual Frame object.
It cannot be used for any computations, is it be used in any structure apart from error structures."""

@typing.final
class LocalFrame:

    def __init__(self):...

    def __int__(self) -> None:
        """int(self)"""

    def __repr__(self) -> str:
        """Return repr(self)."""
    Inertial: LocalFrame = ...
    RCN: LocalFrame = ...
    RIC: LocalFrame = ...
    VNC: LocalFrame = ...

@typing.final
class Location:
    """Location is defined by its latitude, longitude, height above the geoid, mean angular rotation of the geoid, and a frame UID.
If the location includes a terrain mask, it will be used for obstruction checks when computing azimuth and elevation.
**Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s."""
    height_km: float
    latitude_deg: float
    longitude_deg: float
    terrain_mask: list
    terrain_mask_ignored: bool

    def __init__(self, latitude_deg: float, longitude_deg: float, height_km: float, frame: FrameUid, terrain_mask: list, terrain_mask_ignored: bool) -> None:
        """Location is defined by its latitude, longitude, height above the geoid, mean angular rotation of the geoid, and a frame UID.
If the location includes a terrain mask, it will be used for obstruction checks when computing azimuth and elevation.
**Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s."""

    def elevation_mask_at_azimuth_deg(self, azimuth_deg: float) -> float:
        """Returns the elevation mask at the provided azimuth, does NOT account for whether the mask is ignored or not."""

    @staticmethod
    def from_dhall(repr: str) -> Location:
        """Loads a Location from its Dhall representation"""

    def to_dhall(self) -> str:
        """Returns the Dhall representation of this Location"""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class Occultation:
    """Stores the result of an occultation computation with the occultation percentage
Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details."""
    back_frame: Frame
    epoch: Epoch
    front_frame: Frame
    percentage: float

    def __init__(self) -> None:
        """Stores the result of an occultation computation with the occultation percentage
Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details."""

    def factor(self) -> float:
        """Returns the percentage as a factor between 0 and 1"""

    def is_eclipse_computation(self) -> bool:
        """Returns true if the back object is the Sun, false otherwise"""

    def is_obstructed(self) -> bool:
        """Returns true if the occultation percentage is greater than or equal 99.999%"""

    def is_partial(self) -> bool:
        """Returns true if neither occulted nor visible (i.e. penumbra for solar eclipsing)"""

    def is_visible(self) -> bool:
        """Returns true if the occultation percentage is less than or equal 0.001%"""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class Orbit:
    """Defines a Cartesian state in a given frame at a given epoch in a given time scale. Radius data is expressed in kilometers. Velocity data is expressed in kilometers per second.
Regardless of the constructor used, this struct stores all the state information in Cartesian coordinates as these are always non singular.

Unless noted otherwise, algorithms are from GMAT 2016a [StateConversionUtil.cpp](https://github.com/ChristopherRabotin/GMAT/blob/37201a6290e7f7b941bc98ee973a527a5857104b/src/base/util/StateConversionUtil.cpp)."""
    epoch: Epoch
    frame: Frame
    vx_km_s: float
    vy_km_s: float
    vz_km: None
    vz_km_s: float
    x_km: float
    y_km: float
    z_km: float

    def __init__(self, *args: tuples) -> Orbit:
        """Defines a Cartesian state in a given frame at a given epoch in a given time scale. Radius data is expressed in kilometers. Velocity data is expressed in kilometers per second.
Regardless of the constructor used, this struct stores all the state information in Cartesian coordinates as these are always non singular.

Unless noted otherwise, algorithms are from GMAT 2016a [StateConversionUtil.cpp](https://github.com/ChristopherRabotin/GMAT/blob/37201a6290e7f7b941bc98ee973a527a5857104b/src/base/util/StateConversionUtil.cpp)."""

    def abs_difference(self, other: Orbit) -> typing.Tuple:
        """Returns the absolute position and velocity differences in km and km/s between this orbit and another.
Raises an error if the frames do not match (epochs do not need to match)."""

    def abs_pos_diff_km(self, other: Orbit) -> float:
        """Returns the absolute position difference in kilometer between this orbit and another.
Raises an error if the frames do not match (epochs do not need to match)."""

    def abs_vel_diff_km_s(self, other: Orbit) -> float:
        """Returns the absolute velocity difference in kilometer per second between this orbit and another.
Raises an error if the frames do not match (epochs do not need to match)."""

    def add_aop_deg(self, delta_aop_deg: float) -> Orbit:
        """Returns a copy of the state with a provided AOP added to the current one"""

    def add_apoapsis_periapsis_km(self, delta_ra_km: float, delta_rp_km: float) -> Orbit:
        """Returns a copy of this state with the provided apoasis and periapsis added to the current values"""

    def add_ecc(self, delta_ecc: float) -> Orbit:
        """Returns a copy of the state with a provided ECC added to the current one"""

    def add_inc_deg(self, delta_inc_deg: float) -> Orbit:
        """Returns a copy of the state with a provided INC added to the current one"""

    def add_raan_deg(self, delta_raan_deg: float) -> Orbit:
        """Returns a copy of the state with a provided RAAN added to the current one"""

    def add_sma_km(self, delta_sma_km: float) -> Orbit:
        """Returns a copy of the state with a provided SMA added to the current one"""

    def add_ta_deg(self, delta_ta_deg: float) -> Orbit:
        """Returns a copy of the state with a provided TA added to the current one"""

    def altitude_km(self) -> float:
        """Returns the altitude in km"""

    def aol_deg(self) -> float:
        """Returns the argument of latitude in degrees

NOTE: If the orbit is near circular, the AoL will be computed from the true longitude
instead of relying on the ill-defined true anomaly."""

    def aop_brouwer_short_deg(self) -> float:
        """Returns the Brouwer-short mean Argument of Perigee in degrees."""

    def aop_deg(self) -> float:
        """Returns the argument of periapsis in degrees"""

    def apoapsis_altitude_km(self) -> float:
        """Returns the altitude of apoapsis (or apogee around Earth), in kilometers."""

    def apoapsis_km(self) -> float:
        """Returns the radius of apoapsis (or apogee around Earth), in kilometers."""

    def at_epoch(self, new_epoch: Epoch) -> Orbit:
        """Adjusts the true anomaly of this orbit using the mean anomaly.

# Astrodynamics note
This is not a true propagation of the orbit. This is akin to a two body propagation ONLY without any other force models applied.
Use Nyx for high fidelity propagation."""

    def c3_km2_s2(self) -> float:
        """Returns the $C_3$ of this orbit in km^2/s^2"""

    def cartesian_pos_vel(self) -> numpy.array:
        """Returns this state as a Cartesian vector of size 6 in [km, km, km, km/s, km/s, km/s]

Note that the time is **not** returned in the vector."""

    def dcm3x3_from_rcn_to_inertial(self) -> DCM:
        """Builds the rotation matrix that rotates from this state's inertial frame to this state's RCN frame (radial, cross, normal)

# Frame warning
If the stattion is NOT in an inertial frame, then this computation is INVALID.

# Algorithm
1. Compute \\hat{r}, \\hat{h}, the unit vectors of the radius and orbital momentum.
2. Compute the cross product of these
3. Build the DCM with these unit vectors
4. Return the DCM structure"""

    def dcm3x3_from_ric_to_inertial(self) -> DCM:
        """Builds the rotation matrix that rotates from this state's inertial frame to this state's RIC frame

# Frame warning
If the state is NOT in an inertial frame, then this computation is INVALID.

# Algorithm
1. Build the c vector as the normalized orbital momentum vector
2. Build the i vector as the cross product of \\hat{r} and c
3. Build the RIC DCM as a 3x3 of the columns [\\hat{r}, \\hat{i}, \\hat{c}]
4. Return the DCM structure **without** accounting for the transport theorem."""

    def dcm3x3_from_topocentric_to_body_fixed(self) -> DCM:
        """Builds the rotation matrix that rotates from the topocentric frame (SEZ) into the body fixed frame of this state.

# Frame warning
If the state is NOT in a body fixed frame (i.e. ITRF93), then this computation is INVALID.

# Source
From the GMAT MathSpec, page 30 section 2.6.9 and from `Calculate_RFT` in `TopocentricAxes.cpp`, this returns the
rotation matrix from the topocentric frame (SEZ) to body fixed frame.
In the GMAT MathSpec notation, R_{IF} is the DCM from body fixed to inertial. Similarly, R{FT} is from topocentric
to body fixed."""

    def dcm3x3_from_vnc_to_inertial(self) -> DCM:
        """Builds the rotation matrix that rotates from this state's inertial frame to this state's VNC frame (velocity, normal, cross)

# Frame warning
If the stattion is NOT in an inertial frame, then this computation is INVALID.

# Algorithm
1. Compute \\hat{v}, \\hat{h}, the unit vectors of the radius and orbital momentum.
2. Compute the cross product of these
3. Build the DCM with these unit vectors
4. Return the DCM structure."""

    def dcm_from_rcn_to_inertial(self) -> DCM:
        """Builds the rotation matrix that rotates from this state's inertial frame to this state's RCN frame (radial, cross, normal)

# Frame warning
If the stattion is NOT in an inertial frame, then this computation is INVALID.

# Algorithm
1. Compute \\hat{r}, \\hat{h}, the unit vectors of the radius and orbital momentum.
2. Compute the cross product of these
3. Build the DCM with these unit vectors
4. Return the DCM structure with a 6x6 DCM with the time derivative of the VNC frame set.

# Note on the time derivative
If the pre or post states cannot be computed, then the time derivative of the DCM will _not_ be set.
Further note that most astrodynamics tools do *not* account for the time derivative in the RIC frame."""

    def dcm_from_ric_to_inertial(self) -> DCM:
        """Builds the rotation matrix that rotates from this state's inertial frame to this state's RIC frame

# Frame warning
If the state is NOT in an inertial frame, then this computation is INVALID.

# Algorithm
1. Compute the state data one millisecond before and one millisecond assuming two body dynamics
2. Compute the DCM for this state, and the pre and post states
3. Build the c vector as the normalized orbital momentum vector
4. Build the i vector as the cross product of \\hat{r} and c
5. Build the RIC DCM as a 3x3 of the columns [\\hat{r}, \\hat{i}, \\hat{c}], for the post, post, and current states
6. Compute the difference between the DCMs of the pre and post states, to build the DCM time derivative
7. Return the DCM structure with a 6x6 state DCM.

# Note on the time derivative
If the pre or post states cannot be computed, then the time derivative of the DCM will _not_ be set.
Further note that most astrodynamics tools do *not* account for the time derivative in the RIC frame."""

    def dcm_from_topocentric_to_body_fixed(self) -> DCM:
        """Builds the rotation matrix that rotates from the topocentric frame (SEZ) into the body fixed frame of this state.

# Frame warnings
+ If the state is NOT in a body fixed frame (i.e. ITRF93), then this computation is INVALID.
+ (Usually) no time derivative can be computed: the orbit is expected to be a body fixed frame where the `at_epoch` function will fail. Exceptions for Moon body fixed frames.

# UNUSED Arguments
+ `from`: ID of this new frame. Only used to set the "from" frame of the DCM. -- No longer used since 0.5.3

# Source
From the GMAT MathSpec, page 30 section 2.6.9 and from `Calculate_RFT` in `TopocentricAxes.cpp`, this returns the
rotation matrix from the topocentric frame (SEZ) to body fixed frame.
In the GMAT MathSpec notation, R_{IF} is the DCM from body fixed to inertial. Similarly, R{FT} is from topocentric
to body fixed."""

    def dcm_from_vnc_to_inertial(self) -> DCM:
        """Builds the rotation matrix that rotates from this state's inertial frame to this state's VNC frame (velocity, normal, cross)

# Frame warning
If the stattion is NOT in an inertial frame, then this computation is INVALID.

# Algorithm
1. Compute \\hat{v}, \\hat{h}, the unit vectors of the radius and orbital momentum.
2. Compute the cross product of these
3. Build the DCM with these unit vectors
4. Compute the difference between the DCMs of the pre and post states (+/- 1 ms), to build the DCM time derivative
4. Return the DCM structure with a 6x6 DCM with the time derivative of the VNC frame set.

# Note on the time derivative
If the pre or post states cannot be computed, then the time derivative of the DCM will _not_ be set.
Further note that most astrodynamics tools do *not* account for the time derivative in the RIC frame."""

    def dcm_to_inertial(self, local_frame: LocalFrame) -> DCM:
        """Returns the DCM to rotate this orbit from the provided local frame to the inertial frame."""

    def declination_deg(self) -> float:
        """Returns the declination of this orbit in degrees"""

    def distance_to_km(self, other: Orbit) -> float:
        """Returns the distance in kilometers between this state and another state, if both frame match (epoch does not need to match)."""

    def duration_to_radius(self, radius_km: float) -> Duration:
        """Calculates the duration to reach a specific radius in the orbit.

This function computes the time it will take for the orbiting body to reach
the given `radius_km` from its current position. The calculation assumes
two-body dynamics and considers the direction of motion.

# Assumptions & Limitations

- Assumes pure Keplerian motion.
- For elliptical orbits, if the radius is reachable at two points (ascending and descending parts
of the orbit), this function calculates the time to reach the radius corresponding to the
true anomaly in `[0, PI]` (typically the ascending part or up to apoapsis if starting past periapsis).
- For circular orbits, if the radius is within the apoapse and periapse, then a duration of zero is returned.
- For hyperbolic/parabolic orbits, the true anomaly at radius is also computed in `[0, PI]`. If this
point is in the past, the function returns an error, as it doesn't look for solutions on the
departing leg if `nu > PI` would be required (unless current TA is already > PI and target radius is further along).
The current implementation strictly uses the `acos` result, so `nu_rad_at_radius` is always `0 <= nu <= PI`.
This means it finds the time to reach the radius on the path from periapsis up to the point where true anomaly is PI."""

    def ea_deg(self) -> float:
        """Returns the eccentric anomaly in degrees

This is a conversion from GMAT's StateConversionUtil::TrueToEccentricAnomaly"""

    def ecc(self) -> float:
        """Returns the eccentricity (no unit)"""

    def ecc_brouwer_short(self) -> float:
        """Returns the Brouwer-short mean eccentricity."""

    def energy_km2_s2(self) -> float:
        """Returns the specific mechanical energy in km^2/s^2"""

    def eq_within(self, other: Orbit, radial_tol_km: float, velocity_tol_km_s: float) -> bool:
        """Returns whether this orbit and another are equal within the specified radial and velocity absolute tolerances"""

    def equinoctial_a_km(self) -> float:
        """Returns the equinoctial semi-major axis (a) in km."""

    def equinoctial_h(self) -> float:
        """Returns the equinoctial element h (ecc * sin(aop + raan))."""

    def equinoctial_k(self) -> float:
        """Returns the equinoctial element k (ecc * cos(aop + raan))."""

    def equinoctial_lambda_mean_deg(self) -> float:
        """Returns the equinoctial mean longitude (lambda = raan + aop + ma) in degrees."""

    def equinoctial_p(self) -> float:
        """Returns the equinoctial element p (sin(inc/2) * sin(raan))."""

    def equinoctial_q(self) -> float:
        """Returns the equinoctial element q (sin(inc/2) * cos(raan))."""

    def fpa_deg(self) -> float:
        """Returns the flight path angle in degrees"""

    @staticmethod
    def from_cartesian(x_km: float, y_km: float, z_km: float, vx_km_s: float, vy_km_s: float, vz_km_s: float, epoch: Epoch, frame: Frame) -> Orbit:
        """Creates a new Cartesian state in the provided frame at the provided Epoch.

**Units:** km, km, km, km/s, km/s, km/s"""

    @staticmethod
    def from_cartesian_npy(pos_vel: np.array, epoch: Epoch, frame: Frame) -> Orbit:
        """Creates a new Cartesian state from a numpy array, an epoch, and a frame.

**Units:** km, km, km, km/s, km/s, km/s"""

    @staticmethod
    def from_keplerian(sma_km: float, ecc: float, inc_deg: float, raan_deg: float, aop_deg: float, ta_deg: float, epoch: Epoch, frame: Frame) -> Orbit:
        """Creates a new Orbit around the provided Celestial or Geoid frame from the Keplerian orbital elements.

**Units:** km, none, degrees, degrees, degrees, degrees

NOTE: The state is defined in Cartesian coordinates as they are non-singular. This causes rounding
errors when creating a state from its Keplerian orbital elements (cf. the state tests).
One should expect these errors to be on the order of 1e-12."""

    @staticmethod
    def from_keplerian_altitude(sma_altitude_km: float, ecc: float, inc_deg: float, raan_deg: float, aop_deg: float, ta_deg: float, epoch: Epoch, frame: Frame) -> Orbit:
        """Creates a new Orbit from the provided semi-major axis altitude in kilometers"""

    @staticmethod
    def from_keplerian_apsis_altitude(apo_alt_km: float, peri_alt_km: float, inc_deg: float, raan_deg: float, aop_deg: float, ta_deg: float, epoch: Epoch, frame: Frame) -> Orbit:
        """Creates a new Orbit from the provided altitudes of apoapsis and periapsis, in kilometers"""

    @staticmethod
    def from_keplerian_apsis_radii(r_a_km: float, r_p_km: float, inc_deg: float, raan_deg: float, aop_deg: float, ta_deg: float, epoch: Epoch, frame: Frame) -> Orbit:
        """Attempts to create a new Orbit from the provided radii of apoapsis and periapsis, in kilometers"""

    @staticmethod
    def from_keplerian_mean_anomaly(sma_km: float, ecc: float, inc_deg: float, raan_deg: float, aop_deg: float, ma_deg: float, epoch: Epoch, frame: Frame) -> Orbit:
        """Initializes a new orbit from the Keplerian orbital elements using the mean anomaly instead of the true anomaly.

# Implementation notes
This function starts by converting the mean anomaly to true anomaly, and then it initializes the orbit
using the keplerian(..) method.
The conversion is from GMAT's MeanToTrueAnomaly function, transliterated originally by Claude and GPT4 with human adjustments."""

    @staticmethod
    def from_latlongalt(latitude_deg: float, longitude_deg: float, height_km: float, epoch: Epoch, frame: Frame) -> Orbit:
        """Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid, and with ZERO angular velocity in this frame.
Use this initializer for creating a fixed point in the ITRF93 frame for example: the correct angular velocity will be applied when transforming this to EME2000 for example.

Refer to [try_latlongalt_omega] if you need to build a fixed point with a non-zero angular velocity in the definition frame.

NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016"""

    @staticmethod
    def from_latlongalt_omega(latitude_deg: float, longitude_deg: float, height_km: float, angular_velocity_rad_s: np.array, epoch: Epoch, frame: Frame) -> Orbit:
        """Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid given the angular velocity vector.
NOTE: Only specify the angular velocity if there's an EXTRA angular velocity of the lat/long/alt point in the provided frame.

Consider using the [Almanac]'s [angular_velocity_wrt_j2000_rad_s] function or [angular_velocity_rad_s] to retrieve the exact angular velocity vector between two orientations.
Example: build a lat/long/alt point referenced in the ITRF93 frame but by specifying the Frame as the EME2000 frame and providing the angular velocity between the ITRF93 and EME2000 frame at the desired time.

NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016"""

    def height_km(self) -> float:
        """Returns the geodetic height in km.

Reference: Vallado, 4th Ed., Algorithm 12 page 172."""

    def hmag(self) -> float:
        """Returns the norm of the orbital momentum"""

    def hx(self) -> float:
        """Returns the orbital momentum value on the X axis"""

    def hy(self) -> float:
        """Returns the orbital momentum value on the Y axis"""

    def hyperbolic_anomaly_deg(self) -> float:
        """Returns the hyperbolic anomaly in degrees between 0 and 360.0
Returns an error if the orbit is not hyperbolic."""

    def hz(self) -> float:
        """Returns the orbital momentum value on the Z axis"""

    def inc_brouwer_short_deg(self) -> float:
        """Returns the Brouwer-short mean inclination in degrees."""

    def inc_deg(self) -> float:
        """Returns the inclination in degrees"""

    def is_brouwer_short_valid(self) -> bool:
        """Returns whether this state satisfies the requirement to compute the Mean Brouwer Short orbital
element set.

This is a conversion from GMAT's StateConversionUtil::CartesianToBrouwerMeanShort.
The details are at the log level `info`.
NOTE: Mean Brouwer Short are only defined around Earth. However, `nyx` does *not* check the
main celestial body around which the state is defined (GMAT does perform this verification)."""

    def latitude_deg(self) -> float:
        """Returns the geodetic latitude (φ) in degrees. Value is between -180 and +180 degrees.

# Frame warning
This state MUST be in the body fixed frame (e.g. ITRF93) prior to calling this function, or the computation is **invalid**."""

    def latlongalt(self) -> typing.Tuple:
        """Returns the geodetic latitude, geodetic longitude, and geodetic height, respectively in degrees, degrees, and kilometers.

# Algorithm
This uses the Heikkinen procedure, which is not iterative. The results match Vallado and GMAT."""

    def light_time(self) -> Duration:
        """Returns the light time duration between this object and the origin of its reference frame."""

    def longitude_360_deg(self) -> float:
        """Returns the geodetic longitude (λ) in degrees. Value is between 0 and 360 degrees.

# Frame warning
This state MUST be in the body fixed frame (e.g. ITRF93) prior to calling this function, or the computation is **invalid**."""

    def longitude_deg(self) -> float:
        """Returns the geodetic longitude (λ) in degrees. Value is between -180 and 180 degrees.

# Frame warning
This state MUST be in the body fixed frame (e.g. ITRF93) prior to calling this function, or the computation is **invalid**."""

    def ltan_deg(self) -> float:
        """Returns the Longitude of the Ascending Node (LTAN), or an error of equatorial orbits"""

    def ma_brouwer_short_deg(self) -> float:
        """Returns the Brouwer-short mean Mean Anomaly in degrees."""

    def ma_deg(self) -> float:
        """Returns the mean anomaly in degrees

This is a conversion from GMAT's StateConversionUtil::TrueToMeanAnomaly"""

    def mean_motion_deg_s(self) -> float:
        """Returns the mean motion in degrees per seconds"""

    def periapsis_altitude_km(self) -> float:
        """Returns the altitude of periapsis (or perigee around Earth), in kilometers."""

    def periapsis_km(self) -> float:
        """Returns the radius of periapsis (or perigee around Earth), in kilometers."""

    def period(self) -> Duration:
        """Returns the period"""

    def raan_brouwer_short_deg(self) -> float:
        """Returns the Brouwer-short mean Right Ascension of the Ascending Node in degrees."""

    def raan_deg(self) -> float:
        """Returns the right ascension of the ascending node in degrees"""

    def radius_km(self) -> np.array:
        """radius vector in km"""

    def rel_difference(self, other: Orbit) -> typing.Tuple:
        """Returns the relative difference between this orbit and another for the position and velocity, respectively the first and second return values.
Both return values are UNITLESS because the relative difference is computed as the absolute difference divided by the rmag and vmag of this object.
Raises an error if the frames do not match, if the position is zero or the velocity is zero."""

    def rel_pos_diff(self, other: Orbit) -> float:
        """Returns the relative position difference (unitless) between this orbit and another.
This is computed by dividing the absolute difference by the norm of this object's radius vector.
If the radius is zero, this function raises a math error.
Raises an error if the frames do not match or  (epochs do not need to match)."""

    def rel_vel_diff(self, other: Orbit) -> float:
        """Returns the absolute velocity difference in kilometer per second between this orbit and another.
Raises an error if the frames do not match (epochs do not need to match)."""

    def ric_difference(self, other: Orbit) -> Orbit:
        """Returns a Cartesian state representing the RIC difference between self and other, in position and velocity (with transport theorem).
Refer to dcm_from_ric_to_inertial for details on the RIC frame.

# Algorithm
1. Compute the RIC DCM of self
2. Rotate self into the RIC frame
3. Rotation other into the RIC frame
4. Compute the difference between these two states
5. Strip the astrodynamical information from the frame, enabling only computations from `CartesianState`"""

    def right_ascension_deg(self) -> float:
        """Returns the right ascension of this orbit in degrees"""

    def rmag_km(self) -> float:
        """Returns the magnitude of the radius vector in km"""

    def rms_radius_km(self, other: Orbit) -> float:
        """Returns the root sum squared (RMS) radius difference between this state and another state, if both frames match (epoch does not need to match)"""

    def rms_velocity_km_s(self, other: Orbit) -> float:
        """Returns the root sum squared (RMS) velocity difference between this state and another state, if both frames match (epoch does not need to match)"""

    def rss_radius_km(self, other: Orbit) -> float:
        """Returns the root mean squared (RSS) radius difference between this state and another state, if both frames match (epoch does not need to match)"""

    def rss_velocity_km_s(self, other: Orbit) -> float:
        """Returns the root mean squared (RSS) velocity difference between this state and another state, if both frames match (epoch does not need to match)"""

    def semi_minor_axis_km(self) -> float:
        """Returns the semi minor axis in km, includes code for a hyperbolic orbit"""

    def semi_parameter_km(self) -> float:
        """Returns the semi parameter (or semilatus rectum)"""

    def set_aop_deg(self, new_aop_deg: float) -> None:
        """Mutates this orbit to change the AOP"""

    def set_ecc(self, new_ecc: float) -> None:
        """Mutates this orbit to change the ECC"""

    def set_inc_deg(self, new_inc_deg: float) -> None:
        """Mutates this orbit to change the INC"""

    def set_raan_deg(self, new_raan_deg: float) -> None:
        """Mutates this orbit to change the RAAN"""

    def set_sma_km(self, new_sma_km: float) -> None:
        """Mutates this orbit to change the SMA"""

    def set_ta_deg(self, new_ta_deg: float) -> None:
        """Mutates this orbit to change the TA"""

    def sma_altitude_km(self) -> float:
        """Returns the SMA altitude in km"""

    def sma_brouwer_short_km(self) -> float:
        """Returns the Brouwer-short mean semi-major axis in km."""

    def sma_km(self) -> float:
        """Returns the semi-major axis in km"""

    def ta_deg(self) -> float:
        """Returns the true anomaly in degrees between 0 and 360.0

NOTE: This function will emit a warning stating that the TA should be avoided if in a very near circular orbit
Code from <https://github.com/ChristopherRabotin/GMAT/blob/80bde040e12946a61dae90d9fc3538f16df34190/src/gmatutil/util/StateConversionUtil.cpp#L6835>

LIMITATION: For an orbit whose true anomaly is (very nearly) 0.0 or 180.0, this function may return either 0.0 or 180.0 with a very small time increment.
This is due to the precision of the cosine calculation: if the arccosine calculation is out of bounds, the sign of the cosine of the true anomaly is used
to determine whether the true anomaly should be 0.0 or 180.0. **In other words**, there is an ambiguity in the computation in the true anomaly exactly at 180.0 and 0.0."""

    def ta_dot_deg_s(self) -> float:
        """Returns the time derivative of the true anomaly computed as the 360.0 degrees divided by the orbital period (in seconds)."""

    def tlong_deg(self) -> float:
        """Returns the true longitude in degrees"""

    def velocity_declination_deg(self) -> float:
        """Returns the velocity declination of this orbit in degrees"""

    def velocity_km_s(self) -> np.array:
        """velocity vector in km/s"""

    def vinf_periapsis_km(self, turn_angle_degrees: float) -> float:
        """Returns the radius of periapse in kilometers for the provided turn angle of this hyperbolic orbit.
Returns an error if the orbit is not hyperbolic."""

    def vinf_turn_angle_deg(self, periapsis_km: float) -> float:
        """Returns the turn angle in degrees for the provided radius of periapse passage of this hyperbolic orbit
Returns an error if the orbit is not hyperbolic."""

    def vmag_km_s(self) -> float:
        """Returns the magnitude of the velocity vector in km/s"""

    def vnc_difference(self, other: Orbit) -> Orbit:
        """Returns a Cartesian state representing the VNC difference between self and other, in position and velocity (with transport theorem).
Refer to dcm_from_vnc_to_inertial for details on the VNC frame.

# Algorithm
1. Compute the VNC DCM of self
2. Rotate self into the VNC frame
3. Rotation other into the VNC frame
4. Compute the difference between these two states
5. Strip the astrodynamical information from the frame, enabling only computations from `CartesianState`"""

    def with_aop_deg(self, new_aop_deg: float) -> Orbit:
        """Returns a copy of the state with a new AOP"""

    def with_apoapsis_periapsis_km(self, new_ra_km: float, new_rp_km: float) -> Orbit:
        """Returns a copy of this state with the provided apoasis and periapsis"""

    def with_ecc(self, new_ecc: float) -> Orbit:
        """Returns a copy of the state with a new ECC"""

    def with_inc_deg(self, new_inc_deg: float) -> Orbit:
        """Returns a copy of the state with a new INC"""

    def with_raan_deg(self, new_raan_deg: float) -> Orbit:
        """Returns a copy of the state with a new RAAN"""

    def with_sma_km(self, new_sma_km: float) -> Orbit:
        """Returns a copy of the state with a new SMA"""

    def with_ta_deg(self, new_ta_deg: float) -> Orbit:
        """Returns a copy of the state with a new TA"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __getnewargs__(self) -> typing.Tuple:...

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
class TerrainMask:
    """TerrainMask is used to compute obstructions during AER calculations."""
    azimuth_deg: float
    elevation_mask_deg: float

    def __init__(self, azimuth_deg: float, elevation_mask_deg: float) -> None:
        """TerrainMask is used to compute obstructions during AER calculations."""

    @staticmethod
    def from_flat_terrain(elevation_mask_deg: float) -> list:
        """Creates a flat terrain mask with the provided elevation mask in degrees"""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""