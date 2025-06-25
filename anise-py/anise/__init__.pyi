import typing


from anise.astro import AzElRange, Frame, Occultation, Orbit
from anise.time import Epoch, TimeScale, TimeSeries
from anise.rotation import DCM

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

    def transform_many(self, target_frame: Orbit, observer_frame: Frame, time_series: TimeSeries, ab_corr: Aberration=None) -> typing.List[Orbit]:
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