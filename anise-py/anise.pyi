import typing
import numpy

__all__: list = ["time", "analysis", "astro", "constants", "rotation", "utils", "Aberration", "Almanac", "MetaAlmanac", "MetaFile"]

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

    def angular_velocity_deg_s(self, from_frame: Frame, to_frame: Frame, epoch: Epoch) -> numpy.array:
        """Returns the angular velocity vector in deg/s of the from_frame wtr to the to_frame.

This can be used to compute the angular velocity of the Earth ITRF93 frame with respect to the J2000 frame for example."""

    def angular_velocity_rad_s(self, from_frame: Frame, to_frame: Frame, epoch: Epoch) -> numpy.array:
        """Returns the angular velocity vector in rad/s of the from_frame wtr to the to_frame.

This can be used to compute the angular velocity of the Earth ITRF93 frame with respect to the J2000 frame for example."""

    def angular_velocity_wtr_j2000_deg_s(self, from_frame: Frame, epoch: Epoch) -> numpy.array:
        """Returns the angular velocity vector in deg/s of the from_frame wtr to the J2000 frame."""

    def angular_velocity_wtr_j2000_rad_s(self, from_frame: Frame, epoch: Epoch) -> numpy.array:
        """Returns the angular velocity vector in rad/s of the from_frame wtr to the J2000 frame."""

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

    def azimuth_elevation_range_sez_from_location(self, rx: Orbit, location: Location, obstructing_body: Frame=None, ab_corr: Aberration=None) -> AzElRange:
        """Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
receiver state (`rx`) seen from the provided location (as transmitter state, once converted into the SEZ frame of the transmitter.
Refer to [azimuth_elevation_range_sez] for algorithm details.
Location terrain masks are always applied, i.e. if the terrain masks the object, all data is set to f64::NAN, unless specified otherwise in the Location."""

    def azimuth_elevation_range_sez_from_location_id(self, rx: Orbit, location_id: int, obstructing_body: Frame=None, ab_corr: Aberration=None) -> AzElRange:
        """Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
receiver state (`rx`) seen from the location ID (as transmitter state, once converted into the SEZ frame of the transmitter.
Refer to [azimuth_elevation_range_sez] for algorithm details."""

    def azimuth_elevation_range_sez_from_location_name(self, rx: Orbit, location_name: str, obstructing_body: Frame=None, ab_corr: Aberration=None) -> AzElRange:
        """Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
receiver state (`rx`) seen from the location ID (as transmitter state, once converted into the SEZ frame of the transmitter.
Refer to [azimuth_elevation_range_sez] for algorithm details."""

    def azimuth_elevation_range_sez_many(self, rx_tx_states: typing.List[Orbit], obstructing_body: Frame=None, ab_corr: Aberration=None) -> typing.List[AzElRange]:
        """Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
receiver states (first item in tuple) seen from the transmitter state (second item in states tuple), once converted into the SEZ frame of the transmitter.

Note: if any computation fails, the error will be printed to the stderr.
Note: the output AER will be chronologically sorted, regardless of transmitter.

Refer to [azimuth_elevation_range_sez] for details."""

    def beta_angle_deg(self, state: Orbit, ab_corr: Aberration=None) -> float:
        """Computes the Beta angle (β) for a given orbital state, in degrees. A Beta angle of 0° indicates that the orbit plane is edge-on to the Sun, leading to maximum eclipse time. Conversely, a Beta angle of +90° or -90° means the orbit plane is face-on to the Sun, resulting in continuous sunlight exposure and no eclipses.

The Beta angle (β) is defined as the angle between the orbit plane of a spacecraft and the vector from the central body (e.g., Earth) to the Sun. In simpler terms, it measures how much of the time a satellite in orbit is exposed to direct sunlight.
The mathematical formula for the Beta angle is: β=arcsin(h⋅usun\u200b)
Where:
- h is the unit vector of the orbital momentum.
- usun\u200b is the unit vector pointing from the central body to the Sun.

Original code from GMAT, <https://github.com/ChristopherRabotin/GMAT/blob/GMAT-R2022a/src/gmatutil/util/CalculationUtilities.cpp#L209-L219>"""

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

    def describe(self, spk: bool=None, bpc: bool=None, planetary: bool=None, eulerparams: bool=None, time_scale: TimeScale=None, round_time: bool=None) -> None:
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

    def load_from_metafile(self, metafile: MetaFile, autodelete: bool) -> Almanac:
        """Load from the provided MetaFile, downloading it if necessary.
Set autodelete to true to automatically delete lock files. Lock files are important in multi-threaded loads."""

    def occultation(self, back_frame: Frame, front_frame: Frame, observer: Orbit, ab_corr: Aberration=None) -> Occultation:
        """Computes the occultation percentage of the `back_frame` object by the `front_frame` object as seen from the observer, when according for the provided aberration correction.

A zero percent occultation means that the back object is fully visible from the observer.
A 100%  percent occultation means that the back object is fully hidden from the observer because of the front frame (i.e. _umbra_ if the back object is the Sun).
A value in between means that the back object is partially hidden from the observser (i.e. _penumbra_ if the back object is the Sun).
Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details."""

    def rotate(self, from_frame: Frame, to_frame: Frame, epoch: Epoch) -> DCM:
        """Returns the 6x6 DCM needed to rotation the `from_frame` to the `to_frame`.

# Warning
This function only performs the rotation and no translation whatsoever. Use the `transform_from_to` function instead to include rotations.

# Note
This function performs a recursion of no more than twice the MAX_TREE_DEPTH."""

    def rotate_to(self, state: Orbit, observer_frame: Frame) -> Orbit:
        """Rotates the provided Cartesian state into the requested observer frame

**WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_to` function instead to include rotations."""

    def solar_eclipsing(self, eclipsing_frame: Frame, observer: Orbit, ab_corr: Aberration=None) -> Occultation:
        """Computes the solar eclipsing of the observer due to the eclipsing_frame.

This function calls `occultation` where the back object is the Sun in the J2000 frame, and the front object
is the provided eclipsing frame."""

    def solar_eclipsing_many(self, eclipsing_frame: Frame, observers: typing.List[Orbit], ab_corr: Aberration=None) -> typing.List[Occultation]:
        """Computes the solar eclipsing of all the observers due to the eclipsing_frame, computed in parallel under the hood.

Note: if any computation fails, the error will be printed to the stderr.
Note: the output AER will be chronologically sorted, regardless of transmitter.

Refer to [solar_eclipsing] for details."""

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

    def state_of(self, object_id: int, observer: Frame, epoch: Epoch, ab_corr: Aberration=None) -> Orbit:
        """Returns the Cartesian state of the object as seen from the provided observer frame (essentially `spkezr`).

# Note
The units will be those of the underlying ephemeris data (typically km and km/s)"""

    def sun_angle_deg(self, target_id: int, observer_id: int, epoch: Epoch, ab_corr: Aberration) -> float:
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

    def sun_angle_deg_from_frame(self, target: Frame, observer: Frame, epoch: Epoch, ab_corr: Aberration) -> float:
        """Convenience function that calls `sun_angle_deg` with the provided frames instead of the ephemeris ID."""

    def transform(self, target_frame: Frame, observer_frame: Frame, epoch: Epoch, ab_corr: Aberration=None) -> Orbit:
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

    def transform_many(self, target_frame: Frame, observer_frame: Frame, time_series: TimeSeries, ab_corr: Aberration=None) -> typing.List[Orbit]:
        """Returns a chronologically sorted list of the Cartesian states that transform the `from_frame` to the `to_frame` for each epoch of the time series, computed in parallel under the hood.
Note: if any transformation fails, the error will be printed to the stderr.

Refer to [transform] for details."""

    def transform_many_to(self, states: typing.List[Orbit], observer_frame: Frame, ab_corr: Aberration=None) -> typing.List[Orbit]:
        """Returns a chronologically sorted list of the provided states as seen from the observer frame, given the aberration.
Note: if any transformation fails, the error will be printed to the stderr.
Note: the input ordering is lost: the output states will not be in the same order as the input states if these are not chronologically sorted!

Refer to [transform_to] for details."""

    def transform_to(self, state: Orbit, observer_frame: Frame, ab_corr: Aberration=None) -> Orbit:
        """Returns the provided state as seen from the observer frame, given the aberration."""

    def translate(self, target_frame: Frame, observer_frame: Frame, epoch: Epoch, ab_corr: Aberration=None) -> Orbit:
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

    def translate_geometric(self, target_frame: Frame, observer_frame: Frame, epoch: Epoch) -> Orbit:
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
    class FrameUid:
        """A unique frame reference that only contains enough information to build the actual Frame object.
    It cannot be used for any computations, is it be used in any structure apart from error structures."""

    @typing.final
    class Location:
        """Location is defined by its latitude, longitude, height above the geoid, mean angular rotation of the geoid, and a frame UID.
    If the location includes a terrain mask, it will be used for obstruction checks when computing azimuth and elevation.
    **Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s."""
        height_km: float
        latitude_deg: float
        longitude_deg: float
        terrain_mask_ignored: bool

        def elevation_mask_at_azimuth_deg(self, azimuth_deg: float) -> float:
            """Returns the elevation mask at the provided azimuth."""

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

        def altitude_km(self) -> float:
            """Returns the altitude in km"""

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

        def dcm_from_topocentric_to_body_fixed(self, _from: int) -> DCM:
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
        def from_latlongalt(latitude_deg: float, longitude_deg: float, height_km: float, angular_velocity_rad_s: float, epoch: Epoch, frame: Frame) -> Orbit:
            """(Low fidelity) Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid given the angular velocity in rad/s applied entirely on the +Z axis.

    **WARNING**: This function assumes that all the angular velocity is applied on the +Z axis..

    **Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s, or 7.292123516990373e-05 rad/s.

    NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
    Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016"""

        @staticmethod
        def from_latlongalt_omega(latitude_deg: float, longitude_deg: float, height_km: float, angular_velocity_rad_s: np.array, epoch: Epoch, frame: Frame) -> Orbit:
            """Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid given the angular velocity vector (omega).

    Consider using the [Almanac]'s [angular_velocity_wrt_j2000_rad_s] function or [angular_velocity_rad_s] to retrieve the exact angular velocity vector between two orientations.

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

        def ltan_deg(self) -> float:
            """Returns the Longitude of the Ascending Node (LTAN), or an error of equatorial orbits"""

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

    @typing.final
    class TerrainMask:
        """TerrainMask is used to compute obstructions during AER calculations."""
def exec_gui():...

@typing.final
class time:
    import datetime
    import typing

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

        def __init__(self, string_repr: str) -> None:
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

        def decompose(self) -> tuple:
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

        def to_parts(self) -> tuple:
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
        duration: Duration
        time_scale: TimeScale

        def __init__(self, string_repr: str) -> None:
            """Defines a nanosecond-precision Epoch.

    Refer to the appropriate functions for initializing this Epoch from different time scales or representations.

    (Python documentation hints)"""

        def ceil(self, duration: Duration) -> Epoch:
            """Ceils this epoch to the closest provided duration in the TAI time scale

    # Example
    ```
    use hifitime::{Epoch, TimeUnits};

    let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
    assert_eq!(
    e.ceil(1.hours()),
    Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
    );

    // 45 minutes is a multiple of 3 minutes, hence this result
    let e = Epoch::from_gregorian_tai(2022, 10, 3, 17, 44, 29, 898032665);
    assert_eq!(
    e.ceil(3.minutes()),
    Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 45, 0)
    );
    ```"""

        def day_of_month(self) -> int:
            """Returns the number of days since the start of the Gregorian month in the current time scale.

    # Example
    ```
    use hifitime::Epoch;
    let dt = Epoch::from_gregorian_tai_at_midnight(2025, 7, 3);
    assert_eq!(dt.day_of_month(), 3);
    ```"""

        def day_of_year(self) -> float:
            """Returns the number of days since the start of the year."""

        def duration_in_year(self) -> Duration:
            """Returns the duration since the start of the year"""

        def floor(self, duration: Duration) -> Epoch:
            """Floors this epoch to the closest provided duration

    # Example
    ```
    use hifitime::{Epoch, TimeUnits};

    let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
    assert_eq!(
    e.floor(1.hours()),
    Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 0, 0)
    );

    let e = Epoch::from_gregorian_tai(2022, 10, 3, 17, 44, 29, 898032665);
    assert_eq!(
    e.floor(3.minutes()),
    Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 42, 0)
    );
    ```"""

        @staticmethod
        def from_bdt_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def from_bdt_nanoseconds(nanoseconds: float) -> Epoch:
            """Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    This may be useful for time keeping devices that use BDT as a time source."""

        @staticmethod
        def from_bdt_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def from_et_duration(duration_since_j2000: Duration) -> Epoch:
            """Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)"""

        @staticmethod
        def from_et_seconds(seconds_since_j2000: float) -> Epoch:
            """Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)"""

        @staticmethod
        def from_gpst_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def from_gpst_nanoseconds(nanoseconds: float) -> Epoch:
            """Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    This may be useful for time keeping devices that use GPS as a time source."""

        @staticmethod
        def from_gpst_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def from_gregorian(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int, time_scale: TimeScale) -> Epoch:
            """Initialize from the Gregorian parts"""

        @staticmethod
        def from_gregorian_at_midnight(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
            """Initialize from the Gregorian parts, time set to midnight"""

        @staticmethod
        def from_gregorian_at_noon(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
            """Initialize from the Gregorian parts, time set to noon"""

        @staticmethod
        def from_gregorian_utc(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int) -> Epoch:
            """Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    Use maybe_from_gregorian_tai if unsure."""

        @staticmethod
        def from_gst_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def from_gst_nanoseconds(nanoseconds: float) -> Epoch:
            """Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    This may be useful for time keeping devices that use GST as a time source."""

        @staticmethod
        def from_gst_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def from_jde_et(days: float) -> Epoch:
            """Initialize from the JDE days"""

        @staticmethod
        def from_jde_tai(days: float) -> Epoch:
            """Initialize an Epoch from given JDE in TAI time scale"""

        @staticmethod
        def from_jde_tdb(days: float) -> Epoch:
            """Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days"""

        @staticmethod
        def from_jde_utc(days: float) -> Epoch:
            """Initialize an Epoch from given JDE in UTC time scale"""

        @staticmethod
        def from_mjd_tai(days: float) -> Epoch:
            """Initialize an Epoch from given MJD in TAI time scale"""

        @staticmethod
        def from_mjd_utc(days: float) -> Epoch:
            """Initialize an Epoch from given MJD in UTC time scale"""

        @staticmethod
        def from_ptp_duration(duration: Duration) -> Epoch:
            """Initialize an Epoch from the provided IEEE 1588-2008 (PTPv2) duration since TAI midnight 1970 January 01.
    PTP uses the TAI timescale but with the Unix Epoch for compatibility with unix systems."""

        @staticmethod
        def from_ptp_nanoseconds(nanoseconds: int) -> Epoch:
            """Initialize an Epoch from the provided IEEE 1588-2008 (PTPv2) nanoseconds timestamp since TAI midnight 1970 January 01.
    PTP uses the TAI timescale but with the Unix Epoch for compatibility with unix systems."""

        @staticmethod
        def from_ptp_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided IEEE 1588-2008 (PTPv2) second timestamp since TAI midnight 1970 January 01.
    PTP uses the TAI timescale but with the Unix Epoch for compatibility with unix systems."""

        @staticmethod
        def from_qzsst_days(days: float) -> Epoch:
            """Initialize an Epoch from the number of days since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def from_qzsst_nanoseconds(nanoseconds: int) -> Epoch:
            """Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    This may be useful for time keeping devices that use QZSS as a time source."""

        @staticmethod
        def from_qzsst_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def from_tai_days(days: float) -> Epoch:
            """Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight"""

        @staticmethod
        def from_tai_duration(duration: Duration) -> Epoch:
            """Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch."""

        @staticmethod
        def from_tai_parts(centuries: int, nanoseconds: int) -> Epoch:
            """Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch."""

        @staticmethod
        def from_tai_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight"""

        @staticmethod
        def from_tdb_duration(duration_since_j2000: Duration) -> Epoch:
            """Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI."""

        @staticmethod
        def from_tdb_seconds(seconds_j2000: float) -> Epoch:
            """Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
    In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct."""

        @staticmethod
        def from_tt_duration(duration: Duration) -> Epoch:
            """Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

        @staticmethod
        def from_tt_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

        @staticmethod
        def from_unix_milliseconds(milliseconds: float) -> Epoch:
            """Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01."""

        @staticmethod
        def from_unix_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01."""

        @staticmethod
        def from_utc_days(days: float) -> Epoch:
            """Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight"""

        @staticmethod
        def from_utc_seconds(seconds: float) -> Epoch:
            """Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight"""

        @staticmethod
        def fromdatetime(dt: datetime.datetime) -> Epoch:
            """Builds an Epoch in UTC from the provided datetime after timezone correction if any is present."""

        def hours(self) -> int:
            """Returns the hours of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        @staticmethod
        def init_from_bdt_days(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_bdt_days` instead
    Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_bdt_nanoseconds(nanoseconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_bdt_nanoseconds` instead
    Initialize an Epoch from the number of days since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    This may be useful for time keeping devices that use BDT as a time source."""

        @staticmethod
        def init_from_bdt_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_bdt_seconds` instead
    Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
    defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_et_duration(duration_since_j2000: Duration) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_et_duration` instead
    Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)"""

        @staticmethod
        def init_from_et_seconds(seconds_since_j2000: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_et_seconds` instead
    Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)"""

        @staticmethod
        def init_from_gpst_days(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gpst_days` instead
    Initialize an Epoch from the number of days since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_gpst_nanoseconds(nanoseconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gpst_nanoseconds` instead
    Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    This may be useful for time keeping devices that use GPS as a time source."""

        @staticmethod
        def init_from_gpst_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gpst_seconds` instead
    Initialize an Epoch from the number of seconds since the GPS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_gregorian(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int, time_scale: TimeScale) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gregorian` instead
    Initialize from the Gregorian parts"""

        @staticmethod
        def init_from_gregorian_at_midnight(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gregorian_at_midnight` instead
    Initialize from the Gregorian parts, time set to midnight"""

        @staticmethod
        def init_from_gregorian_at_noon(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gregorian_at_noon` instead
    Initialize from the Gregorian parts, time set to noon"""

        @staticmethod
        def init_from_gregorian_utc(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gregorian_utc` instead
    Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
    Use maybe_from_gregorian_tai if unsure."""

        @staticmethod
        def init_from_gst_days(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gst_days` instead
    Initialize an Epoch from the number of days since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_gst_nanoseconds(nanoseconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gst_nanoseconds` instead
    Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
    This may be useful for time keeping devices that use GST as a time source."""

        @staticmethod
        def init_from_gst_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_gst_seconds` instead
    Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
    starting on August 21st 1999 Midnight UT,
    (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

        @staticmethod
        def init_from_jde_et(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_jde_et` instead
    Initialize from the JDE days"""

        @staticmethod
        def init_from_jde_tai(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_jde_tai` instead
    Initialize an Epoch from given JDE in TAI time scale"""

        @staticmethod
        def init_from_jde_tdb(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_jde_tdb` instead
    Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days"""

        @staticmethod
        def init_from_jde_utc(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_jde_utc` instead
    Initialize an Epoch from given JDE in UTC time scale"""

        @staticmethod
        def init_from_mjd_tai(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_mjd_tai` instead
    Initialize an Epoch from given MJD in TAI time scale"""

        @staticmethod
        def init_from_mjd_utc(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_mjd_utc` instead
    Initialize an Epoch from given MJD in UTC time scale"""

        @staticmethod
        def init_from_qzsst_days(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_qzsst_days` instead
    Initialize an Epoch from the number of days since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_qzsst_nanoseconds(nanoseconds: int) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_qzsst_nanoseconds` instead
    Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
    This may be useful for time keeping devices that use QZSS as a time source."""

        @staticmethod
        def init_from_qzsst_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_qzsst_seconds` instead
    Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
    defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

        @staticmethod
        def init_from_tai_days(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tai_days` instead
    Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight"""

        @staticmethod
        def init_from_tai_duration(duration: Duration) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tai_duration` instead
    Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch."""

        @staticmethod
        def init_from_tai_parts(centuries: int, nanoseconds: int) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tai_parts` instead
    Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch."""

        @staticmethod
        def init_from_tai_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tai_seconds` instead
    Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight"""

        @staticmethod
        def init_from_tdb_duration(duration_since_j2000: Duration) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tdb_duration` instead
    Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI."""

        @staticmethod
        def init_from_tdb_seconds(seconds_j2000: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tdb_seconds` instead
    Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
    NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
    In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct."""

        @staticmethod
        def init_from_tt_duration(duration: Duration) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tt_duration` instead
    Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

        @staticmethod
        def init_from_tt_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_tt_seconds` instead
    Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

        @staticmethod
        def init_from_unix_milliseconds(milliseconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_unix_milliseconds` instead
    Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01."""

        @staticmethod
        def init_from_unix_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_unix_seconds` instead
    Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01."""

        @staticmethod
        def init_from_utc_days(days: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_utc_days` instead
    Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight"""

        @staticmethod
        def init_from_utc_seconds(seconds: float) -> Epoch:
            """WARNING: Deprecated since 4.1.1; Use `from_utc_seconds` instead
    Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight"""

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

        def max(self, other: Epoch) -> Epoch:
            """Returns the maximum of the two epochs.

    ```
    use hifitime::Epoch;

    let e0 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 20);
    let e1 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 21);

    assert_eq!(e1, e1.max(e0));
    assert_eq!(e1, e0.max(e1));
    ```

    _Note:_ this uses a pointer to `self` which will be copied immediately because Python requires a pointer."""

        def microseconds(self) -> int:
            """Returns the microseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def milliseconds(self) -> int:
            """Returns the milliseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def min(self, other: Epoch) -> Epoch:
            """Returns the minimum of the two epochs.

    ```
    use hifitime::Epoch;

    let e0 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 20);
    let e1 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 21);

    assert_eq!(e0, e1.min(e0));
    assert_eq!(e0, e0.min(e1));
    ```

    _Note:_ this uses a pointer to `self` which will be copied immediately because Python requires a pointer."""

        def minutes(self) -> int:
            """Returns the minutes of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def month_name(self) -> MonthName:...

        def nanoseconds(self) -> int:
            """Returns the nanoseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def next(self, weekday: Weekday) -> Epoch:
            """Returns the next weekday.

    ```
    use hifitime::prelude::*;

    let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
    assert_eq!(epoch.weekday_utc(), Weekday::Saturday);
    assert_eq!(epoch.next(Weekday::Sunday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 3));
    assert_eq!(epoch.next(Weekday::Monday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 4));
    assert_eq!(epoch.next(Weekday::Tuesday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 5));
    assert_eq!(epoch.next(Weekday::Wednesday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 6));
    assert_eq!(epoch.next(Weekday::Thursday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 7));
    assert_eq!(epoch.next(Weekday::Friday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 8));
    assert_eq!(epoch.next(Weekday::Saturday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 9));
    ```"""

        def next_weekday_at_midnight(self, weekday: Weekday) -> Epoch:...

        def next_weekday_at_noon(self, weekday: Weekday) -> Epoch:...

        def precise_timescale_conversion(self, forward: bool, reference_epoch: Epoch, polynomial: Polynomial, target: TimeScale) -> Epoch:
            """Converts this [Epoch] into targeted [TimeScale] using provided [Polynomial].

    ## Input
    - forward: whether this is forward or backward conversion.
    For example, using GPST-UTC [Polynomial]
    - GPST->UTC is the forward conversion
    - UTC->GPST is the backward conversion
    - reference_epoch: any reference [Epoch] for the provided [Polynomial].

    While we support any time difference, it should remain short in pratice (a day at most, for precise applications).
    - polynomial: that must be valid for this reference [Epoch], used in the equation `a0 + a1*dt + a2*dt² = GPST-UTC` for example.
    - target: targetted [TimeScale] we will transition to.

    Example:
    ```
    use hifitime::{Epoch, TimeScale, Polynomial, Unit};

    // random GPST Epoch for forward conversion to UTC
    let t_gpst = Epoch::from_gregorian(2020, 01, 01, 0, 0, 0, 0, TimeScale::GPST);

    // Let's say we know the GPST-UTC polynomials for that day,
    // They allow precise forward transition from GPST to UTC,
    // and precise backward transition from UTC to GPST.
    let gpst_utc_polynomials = Polynomial::from_constant_offset_nanoseconds(1.0);

    // This is the reference [Epoch] attached to the publication of these polynomials.
    // You should use polynomials that remain valid and were provided recently (usually one day at most).
    // Example: polynomials were published 1 hour ago.
    let gpst_reference = t_gpst - 1.0 * Unit::Hour;

    // Forward conversion (to UTC) GPST - a0 + a1 *dt + a2*dt² = UTC
    let t_utc = t_gpst.precise_timescale_conversion(true, gpst_reference, gpst_utc_polynomials, TimeScale::UTC)
    .unwrap();

    // Verify we did transition to UTC
    assert_eq!(t_utc.time_scale, TimeScale::UTC);

    // Verify the resulting [Epoch] is the coarse GPST->UTC transition + fine correction
    let reversed = t_utc.to_time_scale(TimeScale::GPST) + 1.0 * Unit::Nanosecond;
    assert_eq!(reversed, t_gpst);

    // Apply the backward transition, from t_utc back to t_gpst.
    // The timescale conversion works both ways: (from UTC) GPST = UTC + a0 + a1 *dt + a2*dt²
    let backwards = t_utc.precise_timescale_conversion(false, gpst_reference, gpst_utc_polynomials, TimeScale::GPST)
    .unwrap();

    assert_eq!(backwards, t_gpst);

    // It is important to understand that your reference point does not have to be in the past.
    // The only logic that should prevail is to always minimize interpolation gap.
    // In other words, if you can access future interpolation information that would minimize the data gap, they should prevail.
    // Example: +30' in the future.
    let gpst_reference = t_gpst + 30.0 * Unit::Minute;

    // Forward conversion (to UTC) but using polynomials that were released 1 hour after t_gpst
    let t_utc = t_gpst.precise_timescale_conversion(true, gpst_reference, gpst_utc_polynomials, TimeScale::UTC)
    .unwrap();

    // Verifications
    assert_eq!(t_utc.time_scale, TimeScale::UTC);

    let reversed = t_utc.to_time_scale(TimeScale::GPST) + 1.0 * Unit::Nanosecond;
    assert_eq!(reversed, t_gpst);
    ```"""

        def previous(self, weekday: Weekday) -> Epoch:
            """Returns the next weekday.

    ```
    use hifitime::prelude::*;

    let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
    assert_eq!(epoch.previous(Weekday::Friday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 1));
    assert_eq!(epoch.previous(Weekday::Thursday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 31));
    assert_eq!(epoch.previous(Weekday::Wednesday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 30));
    assert_eq!(epoch.previous(Weekday::Tuesday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 29));
    assert_eq!(epoch.previous(Weekday::Monday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 28));
    assert_eq!(epoch.previous(Weekday::Sunday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 27));
    assert_eq!(epoch.previous(Weekday::Saturday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 26));
    ```"""

        def previous_weekday_at_midnight(self, weekday: Weekday) -> Epoch:...

        def previous_weekday_at_noon(self, weekday: Weekday) -> Epoch:...

        def round(self, duration: Duration) -> Epoch:
            """Rounds this epoch to the closest provided duration in TAI

    # Example
    ```
    use hifitime::{Epoch, TimeUnits};

    let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
    assert_eq!(
    e.round(1.hours()),
    Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
    );
    ```"""

        def seconds(self) -> int:
            """Returns the seconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

        def strftime(self, format_str: str) -> str:
            """Formats the epoch according to the given format string. Supports a subset of C89 and hifitime-specific format codes. Refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for available format options."""

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

        def to_gregorian(self, time_scale: TimeScale=None) -> tuple:
            """Converts the Epoch to the Gregorian parts in the (optionally) provided time scale as (year, month, day, hour, minute, second)."""

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

        def to_tai_parts(self) -> tuple:
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

        def to_time_of_week(self) -> tuple:
            """Converts this epoch into the time of week, represented as a rolling week counter into that time scale
    and the number of nanoseconds elapsed in current week (since closest Sunday midnight).
    This is usually how GNSS receivers describe a timestamp."""

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

        def todatetime(self) -> datetime.datetime:
            """Returns a Python datetime object from this Epoch (truncating the nanoseconds away)"""

        def weekday(self) -> Weekday:
            """Returns weekday (uses the TAI representation for this calculation)."""

        def weekday_in_time_scale(self, time_scale: TimeScale) -> Weekday:
            """Returns the weekday in provided time scale **ASSUMING** that the reference epoch of that time scale is a Monday.
    You _probably_ do not want to use this. You probably either want `weekday()` or `weekday_utc()`.
    Several time scales do _not_ have a reference day that's on a Monday, e.g. BDT."""

        def weekday_utc(self) -> Weekday:
            """Returns weekday in UTC timescale"""

        def with_hms(self, hours: int, minutes: int, seconds: int) -> Epoch:
            """Returns a copy of self where the time is set to the provided hours, minutes, seconds
    Invalid number of hours, minutes, and seconds will overflow into their higher unit.
    Warning: this does _not_ set the subdivisions of second to zero."""

        def with_hms_from(self, other: Epoch) -> Epoch:
            """Returns a copy of self where the hours, minutes, seconds is set to the time of the provided epoch but the
    sub-second parts are kept from the current epoch.

    ```
    use hifitime::prelude::*;

    let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
    let other = other_utc.to_time_scale(TimeScale::TDB);

    assert_eq!(
    epoch.with_hms_from(other),
    Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 13)
    );
    ```"""

        def with_hms_strict(self, hours: int, minutes: int, seconds: int) -> Epoch:
            """Returns a copy of self where the time is set to the provided hours, minutes, seconds
    Invalid number of hours, minutes, and seconds will overflow into their higher unit.
    Warning: this will set the subdivisions of seconds to zero."""

        def with_hms_strict_from(self, other: Epoch) -> Epoch:
            """Returns a copy of self where the time is set to the time of the other epoch but the subseconds are set to zero.

    ```
    use hifitime::prelude::*;

    let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
    let other = other_utc.to_time_scale(TimeScale::TDB);

    assert_eq!(
    epoch.with_hms_strict_from(other),
    Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 0)
    );
    ```"""

        def with_time_from(self, other: Epoch) -> Epoch:
            """Returns a copy of self where all of the time components (hours, minutes, seconds, and sub-seconds) are set to the time of the provided epoch.

    ```
    use hifitime::prelude::*;

    let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
    let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
    // If the other Epoch is in another time scale, it does not matter, it will be converted to the correct time scale.
    let other = other_utc.to_time_scale(TimeScale::TDB);

    assert_eq!(
    epoch.with_time_from(other),
    Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 23)
    );
    ```"""

        def year(self) -> int:
            """Returns the number of Gregorian years of this epoch in the current time scale."""

        def year_days_of_year(self) -> tuple:
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
        """List of leap seconds from <https://data.iana.org/time-zones/data/leap-seconds.list>.
    This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
    The unannoucned leap seconds come from dat.c in the SOFA library."""

        def __init__(self) -> None:
            """List of leap seconds from <https://data.iana.org/time-zones/data/leap-seconds.list>.
    This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
    The unannoucned leap seconds come from dat.c in the SOFA library."""

        def is_up_to_date(self) -> bool:
            """Downloads the latest leap second list from IANA, and returns whether the embedded leap seconds are still up to date

    ```
    use hifitime::leap_seconds::LatestLeapSeconds;

    assert!(LatestLeapSeconds::default().is_up_to_date().unwrap(), "Hifitime needs to update its leap seconds list!");
    ```"""

        def __repr__(self) -> str:
            """Return repr(self)."""

    @typing.final
    class LeapSecondsFile:
        """A leap second provider that uses an IERS formatted leap seconds file."""

        def __init__(self, path: str) -> None:
            """A leap second provider that uses an IERS formatted leap seconds file."""

        def __repr__(self) -> str:
            """Return repr(self)."""

    @typing.final
    class MonthName:
        """Defines Month names, can be initialized either from its variant or its integer (1 for January)."""

        def __init__(self, month: int) -> None:
            """Defines Month names, can be initialized either from its variant or its integer (1 for January)."""

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
    class Polynomial:
        """Interpolation [Polynomial] used for example in [TimeScale]
    maintenance, precise monitoring or conversions.

    (Python documentation hints)"""

        def correction_duration(self, time_interval: Duration) -> Duration:
            """Calculate the correction (as [Duration] once again) from [Self] and given
    the interpolation time interval"""

        @staticmethod
        def from_constant_offset(constant: Duration) -> Polynomial:
            """Create a [Polynomial] structure that is only made of a static offset"""

        @staticmethod
        def from_constant_offset_nanoseconds(nanos: float) -> Polynomial:
            """Create a [Polynomial] structure from a static offset expressed in nanoseconds"""

        @staticmethod
        def from_offset_and_rate(constant: Duration, rate: Duration) -> Polynomial:
            """Create a [Polynomial] structure from both static offset and rate of change:"""

        @staticmethod
        def from_offset_rate_nanoseconds(offset_ns: float, drift_ns_s: float) -> Polynomial:
            """Create a [Polynomial] structure from a static offset and drift, in nanoseconds and nanoseconds.s⁻¹"""

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

        def __str__(self) -> str:
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

        def __init__(self, start: Epoch, end: Epoch, step: Duration, inclusive: bool) -> None:
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
    class Weekday:

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
        Friday: Weekday = ...
        Monday: Weekday = ...
        Saturday: Weekday = ...
        Sunday: Weekday = ...
        Thursday: Weekday = ...
        Tuesday: Weekday = ...
        Wednesday: Weekday = ...

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

@typing.final
class rotation:
    
    import numpy as np
    import numpy
    import typing

    @typing.final
    class DCM:
        """Defines a direction cosine matrix from one frame ID to another frame ID, optionally with its time derivative.
    It provides a number of run-time checks that prevent invalid rotations."""
        from_id: int
        rot_mat: numpy.array
        rot_mat_dt: numpy.array
        to_id: int

        def __init__(self, np_rot_mat: numpy.array, from_id: int, to_id: int, np_rot_mat_dt: numpy.array=None) -> DCM:
            """Defines a direction cosine matrix from one frame ID to another frame ID, optionally with its time derivative.
    It provides a number of run-time checks that prevent invalid rotations."""

        def angular_velocity_deg_s(self) -> np.array:
            """Returns the angular velocity vector in deg/s if a rotation rate is defined."""

        def angular_velocity_rad_s(self) -> np.array:
            """Returns the angular velocity vector in rad/s of this DCM is it has a defined rotation rate."""

        @staticmethod
        def from_identity(from_id: int, to_id: int) -> DCM:
            """Builds an identity rotation."""

        @staticmethod
        def from_r1(angle_rad: float, from_id: int, to_id: int) -> DCM:
            """Returns a rotation matrix for a rotation about the X axis.

    Source: `euler1` function from Baslisk"""

        @staticmethod
        def from_r2(angle_rad: float, from_id: int, to_id: int) -> DCM:
            """Returns a rotation matrix for a rotation about the Y axis.

    Source: `euler2` function from Basilisk"""

        @staticmethod
        def from_r3(angle_rad: float, from_id: int, to_id: int) -> DCM:
            """Returns a rotation matrix for a rotation about the Z axis.

    Source: `euler3` function from Basilisk"""

        def get_state_dcm(self) -> numpy.array:
            """Returns the 6x6 DCM to rotate a state. If the time derivative of this DCM is defined, this 6x6 accounts for the transport theorem.
    Warning: you MUST manually install numpy to call this function."""

        def is_identity(self) -> bool:
            """Returns whether this rotation is identity, checking first the frames and then the rotation matrix (but ignores its time derivative)"""

        def is_valid(self, unit_tol: float, det_tol: float) -> bool:
            """Returns whether the `rot_mat` of this DCM is a valid rotation matrix.
    The criteria for validity are:
    -- The columns of the matrix are unit vectors, within a specified tolerance (unit_tol).
    -- The determinant of the matrix formed by unitizing the columns of the input matrix is 1, within a specified tolerance. This criterion ensures that the columns of the matrix are nearly orthogonal, and that they form a right-handed basis (det_tol).
    [Source: SPICE's rotation.req](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html#Validating%20a%20rotation%20matrix)"""

        def skew_symmetric(self) -> np.array:
            """Returns the skew symmetric matrix if this DCM defines a rotation rate."""

        def to_quaternion(self) -> Quaternion:...

        def transpose(self) -> DCM:
            """Returns the transpose of this DCM"""

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
    class Quaternion:
        """Represents the orientation of a rigid body in three-dimensional space using Euler parameters.

    Euler parameters, also known as unit quaternions, are a set of four parameters `b0`, `b1`, `b2`, and `b3`.
    For clarity, in ANISE, these are denoted `w`, `x`, `y`, `z`.
    They are an extension of the concept of using Euler angles for representing orientations and are
    particularly useful because they avoid gimbal lock and are more compact than rotation matrices.

    # Definitions

    Euler parameters are defined in terms of the axis of rotation and the angle of rotation. If a body
    rotates by an angle `θ` about an axis defined by the unit vector `e = [e1, e2, e3]`, the Euler parameters
    are defined as:

    b0 = cos(θ / 2)
    b1 = e1 * sin(θ / 2)
    b2 = e2 * sin(θ / 2)
    b3 = e3 * sin(θ / 2)

    These parameters have the property that `b0^2 + b1^2 + b2^2 + b3^2 = 1`, which means they represent
    a rotation in SO(3) and can be used to interpolate rotations smoothly.

    # Applications

    In the context of spacecraft mechanics, Euler parameters are often used because they provide a
    numerically stable way to represent the attitude of a spacecraft without the singularities that
    are present with Euler angles.

    # Usage
    Importantly, ANISE prevents the composition of two Euler Parameters if the frames do not match."""
        from_id: int
        to_id: int
        w: float
        x: float
        y: float
        z: float

        def __init__(self, w: float, x: float, y: float, z: float, from_id: int, to_id: int) -> None:
            """Represents the orientation of a rigid body in three-dimensional space using Euler parameters.

    Euler parameters, also known as unit quaternions, are a set of four parameters `b0`, `b1`, `b2`, and `b3`.
    For clarity, in ANISE, these are denoted `w`, `x`, `y`, `z`.
    They are an extension of the concept of using Euler angles for representing orientations and are
    particularly useful because they avoid gimbal lock and are more compact than rotation matrices.

    # Definitions

    Euler parameters are defined in terms of the axis of rotation and the angle of rotation. If a body
    rotates by an angle `θ` about an axis defined by the unit vector `e = [e1, e2, e3]`, the Euler parameters
    are defined as:

    b0 = cos(θ / 2)
    b1 = e1 * sin(θ / 2)
    b2 = e2 * sin(θ / 2)
    b3 = e3 * sin(θ / 2)

    These parameters have the property that `b0^2 + b1^2 + b2^2 + b3^2 = 1`, which means they represent
    a rotation in SO(3) and can be used to interpolate rotations smoothly.

    # Applications

    In the context of spacecraft mechanics, Euler parameters are often used because they provide a
    numerically stable way to represent the attitude of a spacecraft without the singularities that
    are present with Euler angles.

    # Usage
    Importantly, ANISE prevents the composition of two Euler Parameters if the frames do not match."""

        @staticmethod
        def about_x(angle_rad: float, from_id: int, to_id: int) -> Quaternion:
            """Creates an Euler Parameter representing the short way rotation about the X (R1) axis"""

        @staticmethod
        def about_y(angle_rad: float, from_id: int, to_id: int) -> Quaternion:
            """Creates an Euler Parameter representing the short way rotation about the Y (R2) axis"""

        @staticmethod
        def about_z(angle_rad: float, from_id: int, to_id: int) -> Quaternion:
            """Creates an Euler Parameter representing the short way rotation about the Z (R3) axis"""

        def as_vector(self) -> np.array:
            """Returns the data of this EP as a vector."""

        def b_matrix(self) -> np.array:
            """Returns the 4x3 matrix which relates the body angular velocity vector w to the derivative of this Euler Parameter.
    dQ/dt = 1/2 [B(Q)] w"""

        def conjugate(self) -> Quaternion:
            """Compute the conjugate of the quaternion.

    # Note
    Because Euler Parameters are unit quaternions, the inverse and the conjugate are identical."""

        def derivative(self, omega_rad_s: np.array) -> Quaternion:
            """Returns the euler parameter derivative for this EP and the body angular velocity vector w
    dQ/dt = 1/2 [B(Q)] omega_rad_s"""

        def is_zero(self) -> bool:
            """Returns true if the quaternion represents a rotation of zero radians"""

        def normalize(self) -> Quaternion:
            """Normalize the quaternion."""

        def prv(self) -> np.array:
            """Returns the principal rotation vector representation of this Euler Parameter"""

        def scalar_norm(self) -> float:
            """Returns the norm of this Euler Parameter as a scalar."""

        def short(self) -> Quaternion:
            """Returns the short way rotation of this quaternion"""

        def to_dcm(self) -> DCM:
            """Convert this quaterion to a DCM"""

        def uvec_angle_rad(self) -> tuple:
            """Returns the principal line of rotation (a unit vector) and the angle of rotation in radians"""

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
class constants:
        import typing

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
        MOON_ME_DE421_FRAME: Frame = ...
        MOON_ME_DE440_ME421_FRAME: Frame = ...
        MOON_ME_FRAME: Frame = ...
        MOON_PA_DE421_FRAME: Frame = ...
        MOON_PA_DE440_FRAME: Frame = ...
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
        MOON_ME_DE421: int = ...
        MOON_ME_DE440_ME421: int = ...
        MOON_PA: int = ...
        MOON_PA_DE421: int = ...
        MOON_PA_DE440: int = ...

    @typing.final
    class UsualConstants:
        MEAN_EARTH_ANGULAR_VELOCITY_DEG_S: float = ...
        MEAN_MOON_ANGULAR_VELOCITY_DEG_S: float = ...
        SPEED_OF_LIGHT_KM_S: float = ...
