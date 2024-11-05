import typing

_all__: list = ["time", "astro", "utils", "Aberration", "Almanac", "MetaAlmanac", "MetaFile"]

@typing.final
class Aberration:
    """Represents the aberration correction options in ANISE.

In space science and engineering, accurately pointing instruments (like optical cameras or radio antennas) at a target is crucial. This task is complicated by the finite speed of light, necessitating corrections for the apparent position of the target.

This structure holds parameters for aberration corrections applied to a target's position or state vector. These corrections account for the difference between the target's geometric (true) position and its apparent position as observed.

# Rule of tumb
In most Earth orbits, one does _not_ need to provide any aberration corrections. Light time to the target is less than one second (the Moon is about one second away).
In near Earth orbits, e.g. inner solar system, preliminary analysis can benefit from enabling unconverged light time correction. Stellar aberration is probably not required.
For deep space missions, preliminary analysis would likely require both light time correction and stellar aberration. Mission planning and operations will definitely need converged light-time calculations.

For more details, <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/abcorr.html>.

# SPICE Validation

The validation test `validate_jplde_de440s_aberration_lt` checks 101,000 pairs of ephemeris computations and shows that the unconverged Light Time computation matches the SPICE computations almost all the time.
More specifically, the 99th percentile of error is less than 5 meters, the 75th percentile is less than one meter, and the median error is less than 2 millimeters."""
    converged: bool
    stellar: bool
    transmit_mode: bool

    def __init__(self, name: str) -> Aberration:
        """Represents the aberration correction options in ANISE.

In space science and engineering, accurately pointing instruments (like optical cameras or radio antennas) at a target is crucial. This task is complicated by the finite speed of light, necessitating corrections for the apparent position of the target.

This structure holds parameters for aberration corrections applied to a target's position or state vector. These corrections account for the difference between the target's geometric (true) position and its apparent position as observed.

# Rule of tumb
In most Earth orbits, one does _not_ need to provide any aberration corrections. Light time to the target is less than one second (the Moon is about one second away).
In near Earth orbits, e.g. inner solar system, preliminary analysis can benefit from enabling unconverged light time correction. Stellar aberration is probably not required.
For deep space missions, preliminary analysis would likely require both light time correction and stellar aberration. Mission planning and operations will definitely need converged light-time calculations.

For more details, <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/abcorr.html>.

# SPICE Validation

The validation test `validate_jplde_de440s_aberration_lt` checks 101,000 pairs of ephemeris computations and shows that the unconverged Light Time computation matches the SPICE computations almost all the time.
More specifically, the 99th percentile of error is less than 5 meters, the 75th percentile is less than one meter, and the median error is less than 2 millimeters."""

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
class Almanac:
    """An Almanac contains all of the loaded SPICE and ANISE data. It is the context for all computations."""

    def __init__(self, path: str) -> Almanac:
        """An Almanac contains all of the loaded SPICE and ANISE data. It is the context for all computations."""

    def azimuth_elevation_range_sez(self, rx: Orbit, tx: Orbit, obstructing_body: Frame=None, ab_corr: Aberration=None) -> AzElRange:
        """Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
receiver state (`rx`) seen from the transmitter state (`tx`), once converted into the SEZ frame of the transmitter.

# Warning
The obstructing body _should_ be a tri-axial ellipsoid body, e.g. IAU_MOON_FRAME.

# Algorithm
1. If any obstructing_bodies are provided, ensure that none of these are obstructing the line of sight between the receiver and transmitter.
2. Compute the SEZ (South East Zenith) frame of the transmitter.
3. Rotate the receiver position vector into the transmitter SEZ frame.
4. Rotate the transmitter position vector into that same SEZ frame.
5. Compute the range as the norm of the difference between these two position vectors.
6. Compute the elevation, and ensure it is between +/- 180 degrees.
7. Compute the azimuth with a quadrant check, and ensure it is between 0 and 360 degrees."""

    def bpc_domain(self, id: int) -> typing.Tuple:
        """Returns the applicable domain of the request id, i.e. start and end epoch that the provided id has loaded data."""

    def bpc_domains(self) -> typing.Dict:
        """Returns a map of each loaded BPC ID to its domain validity.

# Warning
This function performs a memory allocation."""

    def bpc_summaries(self, id: int) -> typing.List:
        """Returns a vector of the summaries whose ID matches the desired `id`, in the order in which they will be used, i.e. in reverse loading order.

# Warning
This function performs a memory allocation."""

    def describe(self, spk: bool=None, bpc: bool=None, planetary: bool=None, time_scale: TimeScale=None, round_time: bool=None) -> None:
        """Pretty prints the description of this Almanac, showing everything by default. Default time scale is TDB.
If any parameter is set to true, then nothing other than that will be printed."""

    def frame_info(self, uid: Frame) -> Frame:
        """Returns the frame information (gravitational param, shape) as defined in this Almanac from an empty frame"""

    def line_of_sight_obstructed(self, observer: Orbit, observed: Orbit, obstructing_body: Frame, ab_corr: Aberration=None) -> bool:
        """Computes whether the line of sight between an observer and an observed Cartesian state is obstructed by the obstructing body.
Returns true if the obstructing body is in the way, false otherwise.

For example, if the Moon is in between a Lunar orbiter (observed) and a ground station (observer), then this function returns `true`
because the Moon (obstructing body) is indeed obstructing the line of sight.

```text
Observed
o  -
+    -
+      -
+ ***   -
* +    *   -
*  + + * + + o
*     *     Observer
****
```

Key Elements:
- `o` represents the positions of the observer and observed objects.
- The dashed line connecting the observer and observed is the line of sight.

Algorithm (source: Algorithm 35 of Vallado, 4th edition, page 308.):
- `r1` and `r2` are the transformed radii of the observed and observer objects, respectively.
- `r1sq` and `r2sq` are the squared magnitudes of these vectors.
- `r1dotr2` is the dot product of `r1` and `r2`.
- `tau` is a parameter that determines the intersection point along the line of sight.
- The condition `(1.0 - tau) * r1sq + r1dotr2 * tau <= ob_mean_eq_radius_km^2` checks if the line of sight is within the obstructing body's radius, indicating an obstruction."""

    def load(self, path: str) -> Almanac:
        """Generic function that tries to load the provided path guessing to the file type."""

    def load_from_metafile(self, metafile: Metafile, autodelete: bool) -> Almanac:
        """Load from the provided MetaFile, downloading it if necessary.
Set autodelete to true to automatically delete lock files. Lock files are important in multi-threaded loads."""

    def occultation(self, back_frame: Frame, front_frame: Frame, observer: Orbit, ab_corr: Aberration=None) -> Occultation:
        """Computes the occultation percentage of the `back_frame` object by the `front_frame` object as seen from the observer, when according for the provided aberration correction.

A zero percent occultation means that the back object is fully visible from the observer.
A 100%  percent occultation means that the back object is fully hidden from the observer because of the front frame (i.e. _umbra_ if the back object is the Sun).
A value in between means that the back object is partially hidden from the observser (i.e. _penumbra_ if the back object is the Sun).
Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details."""

    def solar_eclipsing(self, eclipsing_frame: Frame, observer: Orbit, ab_corr: Aberration=None) -> Occultation:
        """Computes the solar eclipsing of the observer due to the eclipsing_frame.

This function calls `occultation` where the back object is the Sun in the J2000 frame, and the front object
is the provided eclipsing frame."""

    def spk_domain(self, id: int) -> typing.Tuple:
        """Returns the applicable domain of the request id, i.e. start and end epoch that the provided id has loaded data."""

    def spk_domains(self) -> typing.Dict:
        """Returns a map of each loaded SPK ID to its domain validity.

# Warning
This function performs a memory allocation."""

    def spk_ezr(self, target: int, epoch: Epoch, frame: int, observer: int, ab_corr: Aberration=None) -> Orbit:
        """Alias fo SPICE's `spkezr` where the inputs must be the NAIF IDs of the objects and frames with the caveat that the aberration is moved to the last positional argument."""

    def spk_summaries(self, id: int) -> typing.List:
        """Returns a vector of the summaries whose ID matches the desired `id`, in the order in which they will be used, i.e. in reverse loading order.

# Warning
This function performs a memory allocation."""

    def state_of(self, object: int, observer: Frame, epoch: Epoch, ab_corr: Aberration=None) -> Orbit:
        """Returns the Cartesian state of the object as seen from the provided observer frame (essentially `spkezr`).

# Note
The units will be those of the underlying ephemeris data (typically km and km/s)"""

    def sun_angle_deg(self, target_id: int, observer_id: int, epoch: Epoch) -> float:
        """Returns the angle (between 0 and 180 degrees) between the observer and the Sun, and the observer and the target body ID.
This computes the Sun Probe Earth angle (SPE) if the probe is in a loaded SPK, its ID is the "observer_id", and the target is set to its central body.

# Geometry
If the SPE is greater than 90 degrees, then the celestial object below the probe is in sunlight.

## Sunrise at nadir
```text
Sun
|  \\
|   \\
|    \\
Obs. -- Target
```
## Sun high at nadir
```text
Sun
\\
\\  __ θ > 90
\\     \\
Obs. ---------- Target
```

## Sunset at nadir
```text
Sun
/
/  __ θ < 90
/    /
Obs. -- Target
```

# Algorithm
1. Compute the position of the Sun as seen from the observer
2. Compute the position of the target as seen from the observer
3. Return the arccosine of the dot product of the norms of these vectors."""

    def sun_angle_deg_from_frame(self, target: Frame, observer: Frame, epoch: Epoch) -> float:
        """Convenience function that calls `sun_angle_deg` with the provided frames instead of the ephemeris ID."""

    def transform(self, target_frame: Orbit, observer_frame: Frame, epoch: Epoch, ab_corr: Aberration=None) -> Orbit:
        """Returns the Cartesian state needed to transform the `from_frame` to the `to_frame`.

# SPICE Compatibility
This function is the SPICE equivalent of spkezr: `spkezr(TARGET_ID, EPOCH_TDB_S, ORIENTATION_ID, ABERRATION, OBSERVER_ID)`
In ANISE, the TARGET_ID and ORIENTATION are provided in the first argument (TARGET_FRAME), as that frame includes BOTH
the target ID and the orientation of that target. The EPOCH_TDB_S is the epoch in the TDB time system, which is computed
in ANISE using Hifitime. THe ABERRATION is computed by providing the optional Aberration flag. Finally, the OBSERVER
argument is replaced by OBSERVER_FRAME: if the OBSERVER_FRAME argument has the same orientation as the TARGET_FRAME, then this call
will return exactly the same data as the spkerz SPICE call.

# Note
The units will be those of the underlying ephemeris data (typically km and km/s)"""

    def transform_to(self, state: Orbit, observer_frame: Frame, ab_corr: Aberration=None) -> Orbit:
        """Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame

**WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations."""

    def translate(self, target_frame: Orbit, observer_frame: Frame, epoch: Epoch, ab_corr: Aberration=None) -> Orbit:
        """Returns the Cartesian state of the target frame as seen from the observer frame at the provided epoch, and optionally given the aberration correction.

# SPICE Compatibility
This function is the SPICE equivalent of spkezr: `spkezr(TARGET_ID, EPOCH_TDB_S, ORIENTATION_ID, ABERRATION, OBSERVER_ID)`
In ANISE, the TARGET_ID and ORIENTATION are provided in the first argument (TARGET_FRAME), as that frame includes BOTH
the target ID and the orientation of that target. The EPOCH_TDB_S is the epoch in the TDB time system, which is computed
in ANISE using Hifitime. THe ABERRATION is computed by providing the optional Aberration flag. Finally, the OBSERVER
argument is replaced by OBSERVER_FRAME: if the OBSERVER_FRAME argument has the same orientation as the TARGET_FRAME, then this call
will return exactly the same data as the spkerz SPICE call.

# Warning
This function only performs the translation and no rotation whatsoever. Use the `transform` function instead to include rotations.

# Note
This function performs a recursion of no more than twice the [MAX_TREE_DEPTH]."""

    def translate_geometric(self, target_frame: Orbit, observer_frame: Frame, epoch: Epoch) -> Orbit:
        """Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2."""

    def translate_to(self, state: Orbit, observer_frame: Frame, ab_corr: Aberration=None) -> Orbit:
        """Translates the provided Cartesian state into the requested observer frame

**WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_to` function instead to include rotations."""

    def translate_to_parent(self, source: Frame, epoch: Epoch) -> Orbit:
        """Performs the GEOMETRIC translation to the parent. Use translate_from_to for aberration."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class MetaAlmanac:
    """A structure to set up an Almanac, with automatic downloading, local storage, checksum checking, and more.

# Behavior
If the URI is a local path, relative or absolute, nothing will be fetched from a remote. Relative paths are relative to the execution folder (i.e. the current working directory).
If the URI is a remote path, the MetaAlmanac will first check if the file exists locally. If it exists, it will check that the CRC32 checksum of this file matches that of the specs.
If it does not match, the file will be downloaded again. If no CRC32 is provided but the file exists, then the MetaAlmanac will fetch the remote file and overwrite the existing file.
The downloaded path will be stored in the "AppData" folder."""
    files: typing.List

    def __init__(self, maybe_path: str=None) -> MetaAlmanac:
        """A structure to set up an Almanac, with automatic downloading, local storage, checksum checking, and more.

# Behavior
If the URI is a local path, relative or absolute, nothing will be fetched from a remote. Relative paths are relative to the execution folder (i.e. the current working directory).
If the URI is a remote path, the MetaAlmanac will first check if the file exists locally. If it exists, it will check that the CRC32 checksum of this file matches that of the specs.
If it does not match, the file will be downloaded again. If no CRC32 is provided but the file exists, then the MetaAlmanac will fetch the remote file and overwrite the existing file.
The downloaded path will be stored in the "AppData" folder."""

    def dumps(self) -> str:
        """Dumps the configured Meta Almanac into a Dhall string."""

    @staticmethod
    def latest(autodelete: bool=None) -> MetaAlmanac:
        """Returns an Almanac loaded from the latest NAIF data via the `default` MetaAlmanac.
The MetaAlmanac will download the DE440s.bsp file, the PCK0008.PCA, the full Moon Principal Axis BPC (moon_pa_de440_200625) and the latest high precision Earth kernel from JPL.

# File list
- <http://public-data.nyxspace.com/anise/de440s.bsp>
- <http://public-data.nyxspace.com/anise/v0.5/pck08.pca>
- <http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc>
- <https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc>

# Reproducibility

Note that the `earth_latest_high_prec.bpc` file is regularly updated daily (or so). As such,
if queried at some future time, the Earth rotation parameters may have changed between two queries.

Set `autodelete` to true to delete lock file if a dead lock is detected after 10 seconds."""

    @staticmethod
    def loads(s: str) -> MetaAlmanac:
        """Loads the provided string as a Dhall configuration to build a MetaAlmanac"""

    def process(self, autodelete: bool=None) -> Almanac:
        """Fetch all of the URIs and return a loaded Almanac.
When downloading the data, ANISE will create a temporarily lock file to prevent race conditions
where multiple processes download the data at the same time. Set `autodelete` to true to delete
this lock file if a dead lock is detected after 10 seconds. Set this flag to false if you have
more than ten processes which may attempt to download files in parallel."""

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
class MetaFile:
    """MetaFile allows downloading a remote file from a URL (http, https only), and interpolation of paths in environment variable using the Dhall syntax `env:MY_ENV_VAR`.

The data is stored in the user's local temp directory (i.e. `~/.local/share/nyx-space/anise/` on Linux and `AppData/Local/nyx-space/anise/` on Windows).
Prior to loading a remote resource, if the local resource exists, its CRC32 will be computed: if it matches the CRC32 of this instance of MetaFile,
then the file will not be downloaded a second time."""
    crc32: int
    uri: str

    def __init__(self, uri: str, crc32: int=None) -> MetaFile:
        """MetaFile allows downloading a remote file from a URL (http, https only), and interpolation of paths in environment variable using the Dhall syntax `env:MY_ENV_VAR`.

The data is stored in the user's local temp directory (i.e. `~/.local/share/nyx-space/anise/` on Linux and `AppData/Local/nyx-space/anise/` on Windows).
Prior to loading a remote resource, if the local resource exists, its CRC32 will be computed: if it matches the CRC32 of this instance of MetaFile,
then the file will not be downloaded a second time."""

    def process(self, autodelete: bool=None) -> None:
        """Processes this MetaFile by downloading it if it's a URL.

This function modified `self` and changes the URI to be the path to the downloaded file."""

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
class astro:
    _all__: list = ["constants", "AzElRange", "Ellipsoid", "Occultation", "Orbit"]

    @typing.final
    class AzElRange:
        """A structure that stores the result of Azimuth, Elevation, Range, Range rate calculation."""
        azimuth_deg: float
        elevation_deg: float
        epoch: Epoch
        light_time: Duration
        obstructed_by: Frame
        range_km: float
        range_rate_km_s: float

        def __init__(self, epoch: Epoch, azimuth_deg: float, elevation_deg: float, range_km: float, range_rate_km_s: float, obstructed_by: Frame=None) -> AzElRange:
            """A structure that stores the result of Azimuth, Elevation, Range, Range rate calculation."""

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
    class Occultation:
        """Stores the result of an occultation computation with the occulation percentage
    Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details."""
        back_frame: Frame
        epoch: Epoch
        front_frame: Frame
        percentage: float

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

        def __init__(self, x_km: float, y_km: float, z_km: float, vx_km_s: float, vy_km_s: float, vz_km_s: float, epoch: Epoch, frame: Frame) -> Orbit:
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

        def add_inc_deg(self, delta_inc_deg: float) -> None:
            """Returns a copy of the state with a provided INC added to the current one"""

        def add_raan_deg(self, delta_raan_deg: float) -> Orbit:
            """Returns a copy of the state with a provided RAAN added to the current one"""

        def add_sma_km(self, delta_sma_km: float) -> Orbit:
            """Returns a copy of the state with a provided SMA added to the current one"""

        def add_ta_deg(self, delta_ta_deg: float) -> Orbit:
            """Returns a copy of the state with a provided TA added to the current one"""

        def aol_deg(self) -> float:
            """Returns the argument of latitude in degrees

    NOTE: If the orbit is near circular, the AoL will be computed from the true longitude
    instead of relying on the ill-defined true anomaly."""

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

        def declination_deg(self) -> float:
            """Returns the declination of this orbit in degrees"""

        def distance_to_km(self, other: Orbit) -> float:
            """Returns the distance in kilometers between this state and another state, if both frame match (epoch does not need to match)."""

        def ea_deg(self) -> float:
            """Returns the eccentric anomaly in degrees

    This is a conversion from GMAT's StateConversionUtil::TrueToEccentricAnomaly"""

        def ecc(self) -> float:
            """Returns the eccentricity (no unit)"""

        def energy_km2_s2(self) -> float:
            """Returns the specific mechanical energy in km^2/s^2"""

        def eq_within(self, other: Orbit, radial_tol_km: float, velocity_tol_km_s: float) -> bool:
            """Returns whether this orbit and another are equal within the specified radial and velocity absolute tolerances"""

        def fpa_deg(self) -> float:
            """Returns the flight path angle in degrees"""

        @staticmethod
        def from_cartesian(x_km: float, y_km: float, z_km: float, vx_km_s: float, vy_km_s: float, vz_km_s: float, epoch: Epoch, frame: Frame) -> Orbit:
            """Creates a new Cartesian state in the provided frame at the provided Epoch.

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
        def from_latlongalt(latitude_deg: float, longitude_deg: float, height_km: float, angular_velocity: float, epoch: Epoch, frame: Frame) -> Orbit:
            """Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid given the angular velocity.

    **Units:** degrees, degrees, km, rad/s
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

        def ma_deg(self) -> float:
            """Returns the mean anomaly in degrees

    This is a conversion from GMAT's StateConversionUtil::TrueToMeanAnomaly"""

        def periapsis_altitude_km(self) -> float:
            """Returns the altitude of periapsis (or perigee around Earth), in kilometers."""

        def periapsis_km(self) -> float:
            """Returns the radius of periapsis (or perigee around Earth), in kilometers."""

        def period(self) -> Duration:
            """Returns the period in seconds"""

        def raan_deg(self) -> float:
            """Returns the right ascension of the ascending node in degrees"""

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
    
    class constants:
        @typing.final
        class CelestialObjects:
            EARTH: int = ...
            EARTH_MOON_BARYCENTER: int = ...
            JUPITER: int = ...
            JUPITER_BARYCENTER: int = ...
            MARS: int = ...
            MARS_BARYCENTER: int = ...
            MERCURY: int = ...
            MOON: int = ...
            NEPTUNE: int = ...
            NEPTUNE_BARYCENTER: int = ...
            PLUTO_BARYCENTER: int = ...
            SATURN: int = ...
            SATURN_BARYCENTER: int = ...
            SOLAR_SYSTEM_BARYCENTER: int = ...
            SUN: int = ...
            URANUS: int = ...
            URANUS_BARYCENTER: int = ...
            VENUS: int = ...

        @typing.final
        class Frames:
            EARTH_ECLIPJ2000: Frame = ...
            EARTH_ITRF93: Frame = ...
            EARTH_J2000: Frame = ...
            EARTH_MOON_BARYCENTER_J2000: Frame = ...
            EME2000: Frame = ...
            IAU_EARTH_FRAME: Frame = ...
            IAU_JUPITER_FRAME: Frame = ...
            IAU_MARS_FRAME: Frame = ...
            IAU_MERCURY_FRAME: Frame = ...
            IAU_MOON_FRAME: Frame = ...
            IAU_NEPTUNE_FRAME: Frame = ...
            IAU_SATURN_FRAME: Frame = ...
            IAU_URANUS_FRAME: Frame = ...
            IAU_VENUS_FRAME: Frame = ...
            JUPITER_BARYCENTER_J2000: Frame = ...
            MARS_BARYCENTER_J2000: Frame = ...
            MERCURY_J2000: Frame = ...
            MOON_J2000: Frame = ...
            MOON_ME_FRAME: Frame = ...
            MOON_PA_FRAME: Frame = ...
            NEPTUNE_BARYCENTER_J2000: Frame = ...
            PLUTO_BARYCENTER_J2000: Frame = ...
            SATURN_BARYCENTER_J2000: Frame = ...
            SSB_J2000: Frame = ...
            SUN_J2000: Frame = ...
            URANUS_BARYCENTER_J2000: Frame = ...
            VENUS_J2000: Frame = ...

        @typing.final
        class Orientations:
            ECLIPJ2000: int = ...
            IAU_EARTH: int = ...
            IAU_JUPITER: int = ...
            IAU_MARS: int = ...
            IAU_MERCURY: int = ...
            IAU_MOON: int = ...
            IAU_NEPTUNE: int = ...
            IAU_SATURN: int = ...
            IAU_URANUS: int = ...
            IAU_VENUS: int = ...
            ITRF93: int = ...
            J2000: int = ...
            MOON_ME: int = ...
            MOON_PA: int = ...

        @typing.final
        class UsualConstants:
            MEAN_EARTH_ANGULAR_VELOCITY_DEG_S: float = ...
            MEAN_MOON_ANGULAR_VELOCITY_DEG_S: float = ...
            SPEED_OF_LIGHT_KM_S: float = ...

@typing.final
class time:
    @typing.final
    class Duration:
        """Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.

    **Important conventions:**
    1. The negative durations can be mentally modeled "BC" years. One hours before 01 Jan 0000, it was "-1" years but  365 days and 23h into the current day.
    It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
    a duration with centuries = -1 and nanoseconds = 0 is _a greater duration_ (further from zero) than centuries = -1 and nanoseconds = 1.
    Duration zero minus one nanosecond returns a century of -1 and a nanosecond set to the number of nanoseconds in one century minus one.
    That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
    As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
    2. It was also decided that opposite durations are equal, e.g. -15 minutes == 15 minutes. If the direction of time matters, use the signum function.

    (Python documentation hints)"""

        def __init__(self, string_repr: str) -> Duration:
            """Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.

    **Important conventions:**
    1. The negative durations can be mentally modeled "BC" years. One hours before 01 Jan 0000, it was "-1" years but  365 days and 23h into the current day.
    It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
    a duration with centuries = -1 and nanoseconds = 0 is _a greater duration_ (further from zero) than centuries = -1 and nanoseconds = 1.
    Duration zero minus one nanosecond returns a century of -1 and a nanosecond set to the number of nanoseconds in one century minus one.
    That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
    As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
    2. It was also decided that opposite durations are equal, e.g. -15 minutes == 15 minutes. If the direction of time matters, use the signum function.

    (Python documentation hints)"""

        @staticmethod
        def EPSILON():...

        @staticmethod
        def MAX():...

        @staticmethod
        def MIN():...

        @staticmethod
        def MIN_NEGATIVE():...

        @staticmethod
        def MIN_POSITIVE():...

        @staticmethod
        def ZERO():...

        def abs(self) -> Duration:
            """Returns the absolute value of this duration"""

        def approx(self) -> Duration:
            """Rounds this duration to the largest units represented in this duration.

    This is useful to provide an approximate human duration. Under the hood, this function uses `round`,
    so the "tipping point" of the rounding is half way to the next increment of the greatest unit.
    As shown below, one example is that 35 hours and 59 minutes rounds to 1 day, but 36 hours and 1 minute rounds
    to 2 days because 2 days is closer to 36h 1 min than 36h 1 min is to 1 day.

    # Example

    ```
    use hifitime::{Duration, TimeUnits};

    assert_eq!((2.hours() + 3.minutes()).approx(), 2.hours());
    assert_eq!((24.hours() + 3.minutes()).approx(), 1.days());
    assert_eq!((35.hours() + 59.minutes()).approx(), 1.days());
    assert_eq!((36.hours() + 1.minutes()).approx(), 2.days());
    assert_eq!((47.hours() + 3.minutes()).approx(), 2.days());
    assert_eq!((49.hours() + 3.minutes()).approx(), 2.days());
    ```"""

        def ceil(self, duration: Duration) -> Duration:
            """Ceils this duration to the closest provided duration

    This simply floors then adds the requested duration

    # Example
    ```
    use hifitime::{Duration, TimeUnits};

    let two_hours_three_min = 2.hours() + 3.minutes();
    assert_eq!(two_hours_three_min.ceil(1.hours()), 3.hours());
    assert_eq!(two_hours_three_min.ceil(30.minutes()), 2.hours() + 30.minutes());
    assert_eq!(two_hours_three_min.ceil(4.hours()), 4.hours());
    assert_eq!(two_hours_three_min.ceil(1.seconds()), two_hours_three_min + 1.seconds());
    assert_eq!(two_hours_three_min.ceil(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
    ```"""

        def decompose(self) -> typing.Tuple:
            """Decomposes a Duration in its sign, days, hours, minutes, seconds, ms, us, ns"""

        def floor(self, duration: Duration) -> Duration:
            """Floors this duration to the closest duration from the bottom

    # Example
    ```
    use hifitime::{Duration, TimeUnits};

    let two_hours_three_min = 2.hours() + 3.minutes();
    assert_eq!(two_hours_three_min.floor(1.hours()), 2.hours());
    assert_eq!(two_hours_three_min.floor(30.minutes()), 2.hours());
    // This is zero because we floor by a duration longer than the current duration, rounding it down
    assert_eq!(two_hours_three_min.floor(4.hours()), 0.hours());
    assert_eq!(two_hours_three_min.floor(1.seconds()), two_hours_three_min);
    assert_eq!(two_hours_three_min.floor(1.hours() + 1.minutes()), 2.hours() + 2.minutes());
    assert_eq!(two_hours_three_min.floor(1.hours() + 5.minutes()), 1.hours() + 5.minutes());
    ```"""

        @staticmethod
        def from_all_parts(sign: int, days: int, hours: int, minutes: int, seconds: int, milliseconds: int, microseconds: int, nanoseconds: int) -> Duration:
            """Creates a new duration from its parts"""

        @staticmethod
        def from_parts(centuries: int, nanoseconds: int) -> Duration:
            """Create a normalized duration from its parts"""

        @staticmethod
        def from_total_nanoseconds(nanos: int) -> Duration:
            """Creates a new Duration from its full nanoseconds"""

        def is_negative(self) -> bool:
            """Returns whether this is a negative or positive duration."""

        def max(self, other: Duration) -> Duration:
            """Returns the maximum of the two durations.

    ```
    use hifitime::TimeUnits;

    let d0 = 20.seconds();
    let d1 = 21.seconds();

    assert_eq!(d1, d1.max(d0));
    assert_eq!(d1, d0.max(d1));
    ```"""

        def min(self, other: Duration) -> Duration:
            """Returns the minimum of the two durations.

    ```
    use hifitime::TimeUnits;

    let d0 = 20.seconds();
    let d1 = 21.seconds();

    assert_eq!(d0, d1.min(d0));
    assert_eq!(d0, d0.min(d1));
    ```"""

        def round(self, duration: Duration) -> Duration:
            """Rounds this duration to the closest provided duration

    This performs both a `ceil` and `floor` and returns the value which is the closest to current one.
    # Example
    ```
    use hifitime::{Duration, TimeUnits};

    let two_hours_three_min = 2.hours() + 3.minutes();
    assert_eq!(two_hours_three_min.round(1.hours()), 2.hours());
    assert_eq!(two_hours_three_min.round(30.minutes()), 2.hours());
    assert_eq!(two_hours_three_min.round(4.hours()), 4.hours());
    assert_eq!(two_hours_three_min.round(1.seconds()), two_hours_three_min);
    assert_eq!(two_hours_three_min.round(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
    ```"""

        def signum(self) -> int:
            """Returns the sign of this duration
    + 0 if the number is zero
    + 1 if the number is positive
    + -1 if the number is negative"""

        def to_parts(self) -> typing.Tuple:
            """Returns the centuries and nanoseconds of this duration
    NOTE: These items are not public to prevent incorrect durations from being created by modifying the values of the structure directly."""

        def to_seconds(self) -> float:
            """Returns this duration in seconds f64.
    For high fidelity comparisons, it is recommended to keep using the Duration structure."""

        def to_unit(self, unit: Unit) -> float:...

        def total_nanoseconds(self) -> int:
            """Returns the total nanoseconds in a signed 128 bit integer"""

        def __add__():
            """Return self+value."""

        def __div__():...

        def __eq__(self, value: typing.Any) -> bool:
            """Return self==value."""

        def __ge__(self, value: typing.Any) -> bool:
            """Return self>=value."""

        def __getnewargs__(self):...

        def __gt__(self, value: typing.Any) -> bool:
            """Return self>value."""

        def __le__(self, value: typing.Any) -> bool:
            """Return self<=value."""

        def __lt__(self, value: typing.Any) -> bool:
            """Return self<value."""

        def __mul__():
            """Return self*value."""

        def __ne__(self, value: typing.Any) -> bool:
            """Return self!=value."""

        def __radd__():
            """Return value+self."""

        def __repr__(self) -> str:
            """Return repr(self)."""

        def __rmul__():
            """Return value*self."""

        def __rsub__():
            """Return value-self."""

        def __str__(self) -> str:
            """Return str(self)."""

        def __sub__():
            """Return self-value."""

    @typing.final
    class DurationError:
        __cause__: typing.Any
        __context__: typing.Any
        __suppress_context__: typing.Any
        __traceback__: typing.Any
        args: typing.Any

        def add_note():
            """Exception.add_note(note) --
    add a note to the exception"""

        def with_traceback():
            """Exception.with_traceback(tb) --
    set self.__traceback__ to tb and return self."""

        def __delattr__():
            """Implement delattr(self, name)."""

        def __getattribute__():
            """Return getattr(self, name)."""

        def __init__():
            """Initialize self.  See help(type(self)) for accurate signature."""

        def __repr__():
            """Return repr(self)."""

        def __setattr__():
            """Implement setattr(self, name, value)."""

        def __setstate__():...

        def __str__():
            """Return str(self)."""

    @typing.final
    class Epoch:
        """Defines a nanosecond-precision Epoch.

    Refer to the appropriate functions for initializing this Epoch from different time scales or representations.

    (Python documentation hints)"""

        def __init__(self, string_repr: str) -> Epoch:
            """Defines a nanosecond-precision Epoch.

    Refer to the appropriate functions for initializing this Epoch from different time scales or representations.

    (Python documentation hints)"""

        def day_of_year(self) -> float:
            """Returns the number of days since the start of the year."""

        def duration_in_year(self) -> Duration:
            """Returns the duration since the start of the year"""

        def hours(self) -> int:
            """Returns the hours of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        @staticmethod
        def init_from_bdt_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_bdt_nanoseconds(nanoseconds: float) -> Epoch:
            """Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    This may be useful for time keeping devices that use BDT as a time source."""

        @staticmethod
        def init_from_bdt_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_et_duration(duration_since_j2000: Duration) -> Epoch:
            """Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)"""

        @staticmethod
        def init_from_et_seconds(seconds_since_j2000: float) -> Epoch:
            """Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)"""

        @staticmethod
        def init_from_gpst_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_gpst_nanoseconds(nanoseconds: float) -> Epoch:
            """Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    This may be useful for time keeping devices that use GPS as a time source."""

        @staticmethod
        def init_from_gpst_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_gregorian(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int, time_scale: TimeScale) -> Epoch:
            """Initialize from the Gregorian parts"""

        @staticmethod
        def init_from_gregorian_at_midnight(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
            """Initialize from the Gregorian parts, time set to midnight"""

        @staticmethod
        def init_from_gregorian_at_noon(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
            """Initialize from the Gregorian parts, time set to noon"""

        @staticmethod
        def init_from_gregorian_utc(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int) -> Epoch:
            """Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    Use maybe_from_gregorian_tai if unsure."""

        @staticmethod
        def init_from_gst_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_gst_nanoseconds(nanoseconds: float) -> Epoch:
            """Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    This may be useful for time keeping devices that use GST as a time source."""

        @staticmethod
        def init_from_gst_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_jde_et(days: float) -> Epoch:
            """Initialize from the JDE days"""

        @staticmethod
        def init_from_jde_tai(days: float) -> Epoch:
            """Initialize an Epoch from given JDE in TAI time scale"""

        @staticmethod
        def init_from_jde_tdb(days: float) -> Epoch:
            """Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days"""

        @staticmethod
        def init_from_jde_utc(days: float) -> Epoch:
            """Initialize an Epoch from given JDE in UTC time scale"""

        @staticmethod
        def init_from_mjd_tai(days: float) -> Epoch:
            """Initialize an Epoch from given MJD in TAI time scale"""

        @staticmethod
        def init_from_mjd_utc(days: float) -> Epoch:
            """Initialize an Epoch from given MJD in UTC time scale"""

        @staticmethod
        def init_from_qzsst_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_qzsst_nanoseconds(nanoseconds: int) -> Epoch:
            """Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    This may be useful for time keeping devices that use QZSS as a time source."""

        @staticmethod
        def init_from_qzsst_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_tai_days(days: float) -> Epoch:
            """Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight"""

        @staticmethod
        def init_from_tai_duration(duration: Duration) -> Epoch:
            """Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch."""

        @staticmethod
        def init_from_tai_parts(centuries: int, nanoseconds: int) -> Epoch:
            """Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch."""

        @staticmethod
        def init_from_tai_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight"""

        @staticmethod
        def init_from_tdb_duration(duration_since_j2000: Duration) -> Epoch:
            """Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI."""

        @staticmethod
        def init_from_tdb_seconds(seconds_j2000: float) -> Epoch:
            """Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
    In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct."""

        @staticmethod
        def init_from_tt_duration(duration: Duration) -> Epoch:
            """Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

        @staticmethod
        def init_from_tt_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

        @staticmethod
        def init_from_unix_milliseconds(milliseconds: float) -> Epoch:
            """Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01."""

        @staticmethod
        def init_from_unix_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01."""

        @staticmethod
        def init_from_utc_days(days: float) -> Epoch:
            """Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight"""

        @staticmethod
        def init_from_utc_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight"""

        def isoformat(self) -> str:
            """Equivalent to `datetime.isoformat`, and truncated to 23 chars, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options"""

        def leap_seconds(self, iers_only: bool) -> float:
            """Get the accumulated number of leap seconds up to this Epoch accounting only for the IERS leap seconds and the SOFA scaling from 1960 to 1972, depending on flag.
    Returns None if the epoch is before 1960, year at which UTC was defined.

    # Why does this function return an `Option` when the other returns a value
    This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960."""

        def leap_seconds_iers(self) -> int:
            """Get the accumulated number of leap seconds up to this Epoch accounting only for the IERS leap seconds."""

        def leap_seconds_with_file(self, iers_only: bool, provider: LeapSecondsFile) -> float:
            """Get the accumulated number of leap seconds up to this Epoch from the provided LeapSecondProvider.
    Returns None if the epoch is before 1960, year at which UTC was defined.

    # Why does this function return an `Option` when the other returns a value
    This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960."""

        def microseconds(self) -> int:
            """Returns the microseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def milliseconds(self) -> int:
            """Returns the milliseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def minutes(self) -> int:
            """Returns the minutes of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def month_name(self) -> MonthName:...

        def nanoseconds(self) -> int:
            """Returns the nanoseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def seconds(self) -> int:
            """Returns the seconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def strftime(self, format_str: str) -> str:
            """Equivalent to `datetime.strftime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options"""

        @staticmethod
        def strptime(epoch_str: str, format_str: str) -> Epoch:
            """Equivalent to `datetime.strptime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options"""

        @staticmethod
        def system_now() -> Epoch:
            """Returns the computer clock in UTC"""

        def timedelta(self, other: Duration) -> Duration:
            """Differences between two epochs"""

        def to_bdt_days(self) -> float:
            """Returns days past BDT (BeiDou) Time Epoch, defined as Jan 01 2006 UTC
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        def to_bdt_duration(self) -> Duration:
            """Returns `Duration` past BDT (BeiDou) time Epoch."""

        def to_bdt_nanoseconds(self) -> int:
            """Returns nanoseconds past BDT (BeiDou) Time Epoch, defined as Jan 01 2006 UTC
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    NOTE: This function will return an error if the centuries past GST time are not zero."""

        def to_bdt_seconds(self) -> float:
            """Returns seconds past BDT (BeiDou) Time Epoch"""

        def to_duration_in_time_scale(self, ts: TimeScale) -> Duration:
            """Returns this epoch with respect to the provided time scale.
    This is needed to correctly perform duration conversions in dynamical time scales (e.g. TDB)."""

        def to_et_centuries_since_j2000(self) -> float:
            """Returns the number of centuries since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)"""

        def to_et_days_since_j2000(self) -> float:
            """Returns the number of days since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)"""

        def to_et_duration(self) -> Duration:
            """Returns the duration between J2000 and the current epoch as per NAIF SPICE.

    # Warning
    The et2utc function of NAIF SPICE will assume that there are 9 leap seconds before 01 JAN 1972,
    as this date introduces 10 leap seconds. At the time of writing, this does _not_ seem to be in
    line with IERS and the documentation in the leap seconds list.

    In order to match SPICE, the as_et_duration() function will manually get rid of that difference."""

        def to_et_seconds(self) -> float:
            """Returns the Ephemeris Time seconds past 2000 JAN 01 midnight, matches NASA/NAIF SPICE."""

        def to_gpst_days(self) -> float:
            """Returns days past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        def to_gpst_duration(self) -> Duration:
            """Returns `Duration` past GPS time Epoch."""

        def to_gpst_nanoseconds(self) -> int:
            """Returns nanoseconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    NOTE: This function will return an error if the centuries past GPST time are not zero."""

        def to_gpst_seconds(self) -> float:
            """Returns seconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        def to_gst_days(self) -> float:
            """Returns days past GST (Galileo) Time Epoch,
    starting on August 21st 1999 Midnight UT
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        def to_gst_duration(self) -> Duration:
            """Returns `Duration` past GST (Galileo) time Epoch."""

        def to_gst_nanoseconds(self) -> int:
            """Returns nanoseconds past GST (Galileo) Time Epoch, starting on August 21st 1999 Midnight UT
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    NOTE: This function will return an error if the centuries past GST time are not zero."""

        def to_gst_seconds(self) -> float:
            """Returns seconds past GST (Galileo) Time Epoch"""

        def to_isoformat(self) -> str:
            """The standard ISO format of this epoch (six digits of subseconds) in the _current_ time scale, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options."""

        def to_jde_et(self, unit: Unit) -> float:...

        def to_jde_et_days(self) -> float:
            """Returns the Ephemeris Time JDE past epoch"""

        def to_jde_et_duration(self) -> Duration:...

        def to_jde_tai(self, unit: Unit) -> float:
            """Returns the Julian Days from epoch 01 Jan -4713 12:00 (noon) in desired Duration::Unit"""

        def to_jde_tai_days(self) -> float:
            """Returns the Julian days from epoch 01 Jan -4713, 12:00 (noon)
    as explained in "Fundamentals of astrodynamics and applications", Vallado et al.
    4th edition, page 182, and on [Wikipedia](https://en.wikipedia.org/wiki/Julian_day)."""

        def to_jde_tai_duration(self) -> Duration:
            """Returns the Julian Days from epoch 01 Jan -4713 12:00 (noon) as a Duration"""

        def to_jde_tai_seconds(self) -> float:
            """Returns the Julian seconds in TAI."""

        def to_jde_tdb_days(self) -> float:
            """Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)"""

        def to_jde_tdb_duration(self) -> Duration:...

        def to_jde_tt_days(self) -> float:
            """Returns days past Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

        def to_jde_tt_duration(self) -> Duration:...

        def to_jde_utc_days(self) -> float:
            """Returns the Julian days in UTC."""

        def to_jde_utc_duration(self) -> Duration:
            """Returns the Julian days in UTC as a `Duration`"""

        def to_jde_utc_seconds(self) -> float:
            """Returns the Julian Days in UTC seconds."""

        def to_mjd_tai(self, unit: Unit) -> float:
            """Returns this epoch as a duration in the requested units in MJD TAI"""

        def to_mjd_tai_days(self) -> float:
            """`as_mjd_days` creates an Epoch from the provided Modified Julian Date in days as explained
    [here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight."""

        def to_mjd_tai_seconds(self) -> float:
            """Returns the Modified Julian Date in seconds TAI."""

        def to_mjd_tt_days(self) -> float:
            """Returns days past Modified Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

        def to_mjd_tt_duration(self) -> Duration:...

        def to_mjd_utc(self, unit: Unit) -> float:
            """Returns the Modified Julian Date in the provided unit in UTC."""

        def to_mjd_utc_days(self) -> float:
            """Returns the Modified Julian Date in days UTC."""

        def to_mjd_utc_seconds(self) -> float:
            """Returns the Modified Julian Date in seconds UTC."""

        def to_nanoseconds_in_time_scale(self, time_scale: TimeScale) -> int:
            """Attempts to return the number of nanoseconds since the reference epoch of the provided time scale.
    This will return an overflow error if more than one century has past since the reference epoch in the provided time scale.
    If this is _not_ an issue, you should use `epoch.to_duration_in_time_scale().to_parts()` to retrieve both the centuries and the nanoseconds
    in that century."""

        def to_qzsst_days(self) -> float:
            """Returns days past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        def to_qzsst_duration(self) -> Duration:
            """Returns `Duration` past QZSS time Epoch."""

        def to_qzsst_nanoseconds(self) -> int:
            """Returns nanoseconds past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    NOTE: This function will return an error if the centuries past QZSST time are not zero."""

        def to_qzsst_seconds(self) -> float:
            """Returns seconds past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        def to_rfc3339(self) -> str:
            """Returns this epoch in UTC in the RFC3339 format"""

        def to_tai(self, unit: Unit) -> float:
            """Returns the epoch as a floating point value in the provided unit"""

        def to_tai_days(self) -> float:
            """Returns the number of days since J1900 in TAI"""

        def to_tai_duration(self) -> Duration:
            """Returns this time in a Duration past J1900 counted in TAI"""

        def to_tai_parts(self) -> typing.Tuple:
            """Returns the TAI parts of this duration"""

        def to_tai_seconds(self) -> float:
            """Returns the number of TAI seconds since J1900"""

        def to_tdb_centuries_since_j2000(self) -> float:
            """Returns the number of centuries since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)"""

        def to_tdb_days_since_j2000(self) -> float:
            """Returns the number of days since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)"""

        def to_tdb_duration(self) -> Duration:
            """Returns the Dynamics Barycentric Time (TDB) as a high precision Duration since J2000

    ## Algorithm
    Given the embedded sine functions in the equation to compute the difference between TDB and TAI from the number of TDB seconds
    past J2000, one cannot solve the revert the operation analytically. Instead, we iterate until the value no longer changes.

    1. Assume that the TAI duration is in fact the TDB seconds from J2000.
    2. Offset to J2000 because `Epoch` stores everything in the J1900 but the TDB duration is in J2000.
    3. Compute the offset `g` due to the TDB computation with the current value of the TDB seconds (defined in step 1).
    4. Subtract that offset to the latest TDB seconds and store this as a new candidate for the true TDB seconds value.
    5. Compute the difference between this candidate and the previous one. If the difference is less than one nanosecond, stop iteration.
    6. Set the new candidate as the TDB seconds since J2000 and loop until step 5 breaks the loop, or we've done five iterations.
    7. At this stage, we have a good approximation of the TDB seconds since J2000.
    8. Reverse the algorithm given that approximation: compute the `g` offset, compute the difference between TDB and TAI, add the TT offset (32.184 s), and offset by the difference between J1900 and J2000."""

        def to_tdb_seconds(self) -> float:
            """Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)"""

        def to_time_scale(self, ts: TimeScale) -> Epoch:
            """Converts self to another time scale

    As per the [Rust naming convention](https://rust-lang.github.io/api-guidelines/naming.html#ad-hoc-conversions-follow-as_-to_-into_-conventions-c-conv),
    this borrows an Epoch and returns an owned Epoch."""

        def to_tt_centuries_j2k(self) -> float:
            """Returns the centuries passed J2000 TT"""

        def to_tt_days(self) -> float:
            """Returns days past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

        def to_tt_duration(self) -> Duration:
            """Returns `Duration` past TAI epoch in Terrestrial Time (TT)."""

        def to_tt_seconds(self) -> float:
            """Returns seconds past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

        def to_tt_since_j2k(self) -> Duration:
            """Returns the duration past J2000 TT"""

        def to_unix(self, unit: Unit) -> float:
            """Returns the duration since the UNIX epoch in the provided unit."""

        def to_unix_days(self) -> float:
            """Returns the number days since the UNIX epoch defined 01 Jan 1970 midnight UTC."""

        def to_unix_duration(self) -> Duration:
            """Returns the Duration since the UNIX epoch UTC midnight 01 Jan 1970."""

        def to_unix_milliseconds(self) -> float:
            """Returns the number milliseconds since the UNIX epoch defined 01 Jan 1970 midnight UTC."""

        def to_unix_seconds(self) -> float:
            """Returns the number seconds since the UNIX epoch defined 01 Jan 1970 midnight UTC."""

        def to_utc(self, unit: Unit) -> float:
            """Returns the number of UTC seconds since the TAI epoch"""

        def to_utc_days(self) -> float:
            """Returns the number of UTC days since the TAI epoch"""

        def to_utc_duration(self) -> Duration:
            """Returns this time in a Duration past J1900 counted in UTC"""

        def to_utc_seconds(self) -> float:
            """Returns the number of UTC seconds since the TAI epoch"""

        def year(self) -> int:
            """Returns the number of Gregorian years of this epoch in the current time scale."""

        def year_days_of_year(self) -> typing.Tuple:
            """Returns the year and the days in the year so far (days of year)."""

        def __add__():
            """Return self+value."""

        def __eq__(self, value: typing.Any) -> bool:
            """Return self==value."""

        def __ge__(self, value: typing.Any) -> bool:
            """Return self>=value."""

        def __getnewargs__(self):...

        def __gt__(self, value: typing.Any) -> bool:
            """Return self>value."""

        def __le__(self, value: typing.Any) -> bool:
            """Return self<=value."""

        def __lt__(self, value: typing.Any) -> bool:
            """Return self<value."""

        def __ne__(self, value: typing.Any) -> bool:
            """Return self!=value."""

        def __radd__():
            """Return value+self."""

        def __repr__(self) -> str:
            """Return repr(self)."""

        def __rsub__():
            """Return value-self."""

        def __str__(self) -> str:
            """Return str(self)."""

        def __sub__():
            """Return self-value."""

    @typing.final
    class HifitimeError:
        __cause__: typing.Any
        __context__: typing.Any
        __suppress_context__: typing.Any
        __traceback__: typing.Any
        args: typing.Any

        def add_note():
            """Exception.add_note(note) --
    add a note to the exception"""

        def with_traceback():
            """Exception.with_traceback(tb) --
    set self.__traceback__ to tb and return self."""

        def __delattr__():
            """Implement delattr(self, name)."""

        def __getattribute__():
            """Return getattr(self, name)."""

        def __init__():
            """Initialize self.  See help(type(self)) for accurate signature."""

        def __repr__():
            """Return repr(self)."""

        def __setattr__():
            """Implement setattr(self, name, value)."""

        def __setstate__():...

        def __str__():
            """Return str(self)."""

    @typing.final
    class LatestLeapSeconds:
        """List of leap seconds from https://www.ietf.org/timezones/data/leap-seconds.list .
    This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
    The unannoucned leap seconds come from dat.c in the SOFA library."""

        def __init__(self) -> None:
            """List of leap seconds from https://www.ietf.org/timezones/data/leap-seconds.list .
    This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
    The unannoucned leap seconds come from dat.c in the SOFA library."""

        def __repr__(self) -> str:
            """Return repr(self)."""

    @typing.final
    class LeapSecondsFile:
        """A leap second provider that uses an IERS formatted leap seconds file.

    (Python documentation hints)"""

        def __init__(self, path: str) -> LeapSecondsFile:
            """A leap second provider that uses an IERS formatted leap seconds file.

    (Python documentation hints)"""

        def __repr__(self) -> str:
            """Return repr(self)."""

    @typing.final
    class MonthName:

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
        April: MonthName = ...
        August: MonthName = ...
        December: MonthName = ...
        February: MonthName = ...
        January: MonthName = ...
        July: MonthName = ...
        June: MonthName = ...
        March: MonthName = ...
        May: MonthName = ...
        November: MonthName = ...
        October: MonthName = ...
        September: MonthName = ...

    @typing.final
    class ParsingError:
        __cause__: typing.Any
        __context__: typing.Any
        __suppress_context__: typing.Any
        __traceback__: typing.Any
        args: typing.Any

        def add_note():
            """Exception.add_note(note) --
    add a note to the exception"""

        def with_traceback():
            """Exception.with_traceback(tb) --
    set self.__traceback__ to tb and return self."""

        def __delattr__():
            """Implement delattr(self, name)."""

        def __getattribute__():
            """Return getattr(self, name)."""

        def __init__():
            """Initialize self.  See help(type(self)) for accurate signature."""

        def __repr__():
            """Return repr(self)."""

        def __setattr__():
            """Implement setattr(self, name, value)."""

        def __setstate__():...

        def __str__():
            """Return str(self)."""

    @typing.final
    class TimeScale:
        """Enum of the different time systems available"""

        def uses_leap_seconds(self) -> bool:
            """Returns true if self takes leap seconds into account"""

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
        BDT: TimeScale = ...
        ET: TimeScale = ...
        GPST: TimeScale = ...
        GST: TimeScale = ...
        QZSST: TimeScale = ...
        TAI: TimeScale = ...
        TDB: TimeScale = ...
        TT: TimeScale = ...
        UTC: TimeScale = ...

    @typing.final
    class TimeSeries:
        """An iterator of a sequence of evenly spaced Epochs.

    (Python documentation hints)"""

        def __init__(self, start: Epoch, end: Epoch, step: Duration, inclusive: bool) -> TimeSeries:
            """An iterator of a sequence of evenly spaced Epochs.

    (Python documentation hints)"""

        def __eq__(self, value: typing.Any) -> bool:
            """Return self==value."""

        def __ge__(self, value: typing.Any) -> bool:
            """Return self>=value."""

        def __getnewargs__(self):...

        def __gt__(self, value: typing.Any) -> bool:
            """Return self>value."""

        def __iter__(self) -> typing.Any:
            """Implement iter(self)."""

        def __le__(self, value: typing.Any) -> bool:
            """Return self<=value."""

        def __lt__(self, value: typing.Any) -> bool:
            """Return self<value."""

        def __ne__(self, value: typing.Any) -> bool:
            """Return self!=value."""

        def __next__(self) -> typing.Any:
            """Implement next(self)."""

        def __repr__(self) -> str:
            """Return repr(self)."""

        def __str__(self) -> str:
            """Return str(self)."""

    @typing.final
    class Unit:
        """An Enum to perform time unit conversions."""

        def from_seconds(self):...

        def in_seconds(self):...

        def __add__():
            """Return self+value."""

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

        def __mul__():
            """Return self*value."""

        def __ne__(self, value: typing.Any) -> bool:
            """Return self!=value."""

        def __radd__():
            """Return value+self."""

        def __repr__(self) -> str:
            """Return repr(self)."""

        def __rmul__():
            """Return value*self."""

        def __rsub__():
            """Return value-self."""

        def __sub__():
            """Return self-value."""
        Century: Unit = ...
        Day: Unit = ...
        Hour: Unit = ...
        Microsecond: Unit = ...
        Millisecond: Unit = ...
        Minute: Unit = ...
        Nanosecond: Unit = ...
        Second: Unit = ...
        Week: Unit = ...

    @typing.final
    class Ut1Provider:
        """A structure storing all of the TAI-UT1 data"""

        def __init__(self) -> None:
            """A structure storing all of the TAI-UT1 data"""

        def __repr__(self) -> str:
            """Return repr(self)."""

@typing.final
class utils:

    @staticmethod
    def convert_fk(fk_file_path: str, anise_output_path: str, show_comments: bool=None, overwrite: bool=None) -> None:
        """Converts a KPL/FK file, that defines frame constants like fixed rotations, and frame name to ID mappings into the EulerParameterDataSet equivalent ANISE file.
KPL/FK files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE."""

    @staticmethod
    def convert_tpc(pck_file_path: str, gm_file_path: str, anise_output_path: str, overwrite: bool=None) -> None:
        """Converts two KPL/TPC files, one defining the planetary constants as text, and the other defining the gravity parameters, into the PlanetaryDataSet equivalent ANISE file.
KPL/TPC files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE."""
    __all__: list = ...
    __name__: str = ...

