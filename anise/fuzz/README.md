# Cargo Fuzz

cargo-fuzz is a development focused subcommand for fuzz testing with libFuzzer. For more details, refer to https://github.com/rust-fuzz/cargo-fuzz.


## Setup

`cargo-fuzz` requires a nightly version of the Rust compiler, as it is dependent on `libfuzzer-sys`. Please consider how this can affect your system before continuing.

### Installation

Start by installing by running `cargo install cargo-fuzz`.

### Setup Nightly Compiler

Install the nightly toolchain with `rustup toolchain install nightly`.

Set the nightly toolcahin as the default for _your current working directory_ with `rustup override set nightly`.


## Running

To run the fuzz tests, simply run `cargo fuzz run <fuzz-test-name>`.

If you want to limit the total runtime you can append a `-- -max_total_time=<seconds>` to the run command. You'll want to a `cargo fuzz build <fuzz-test-name>` first to avoid it impacting the max time.


# Legacy Info - To be Deleted (frames, math, and orientation)
## anise/src/frames
### frame.rs
- `from_name(center: &str, ref_frame: &str) -> Result<Self, AlmanacError>`
- `with_ellipsoid(mut self, shape: Ellipsoid) -> Self`

### frameuid.rs
Anything testable appears to be low priority.

## anise/src/math
### interpolation/chebyshev.rs
- `pub fn chebyshev_eval(normalized_time: f64, spline_coeffs: &[f64], spline_radius_s: f64, eval_epoch: Epoch, degree: usize) -> Result<(f64, f64), InterpolationError>`
- `pub fn chebyshev_eval_poly(normalized_time: f64, spline_coeffs: &[f64], eval_epoch: Epoch, degree: usize) -> Result<f64, InterpolationError>`

### interpolation/hermite.rs
- `pub fn hermite_eval(xs: &[f64], ys: &[f64], ydots: &[f64], x_eval: f64) -> Result<(f64, f64), InterpolationError>`

### interpolation/lagrange.rs
- `pub fn lagrange_eval(xs: &[f64], ys: &[f64], x_eval: f64) -> Result<(f64, f64), InterpolationError>`

### rotation/dcm_py.rs
Anything testable appears to be low priority.

### rotation/dcm.rs
- `pub fn state_dcm(&self) -> Matrix6`
- `pub(crate) fn mul_unchecked(&self, other: Self) -> Self`
- `pub fn transpose(&self) -> Self`
- `pub fn is_valid(&self, unit_tol: f64, det_tol: f64) -> bool`
- `impl Mul<&CartesianState> for DCM`
- `impl From<DCM> for Quaternion`
- `impl From<Quaternion> for DCM`

### rotation/quaternion.rs
- `pub fn normalize(&self) -> Self`
- `pub fn b_matrix(&self) -> Matrix4x3<f64>`
- `pub fn derivative(&self, w: Vector3) -> Self`
- `pub fn derivative(&self, w: Vector3) -> Self`
- `pub fn uvec_angle(&self) -> (Vector3, f64)`
- `pub fn prv(&self) -> Vector3`

### angles.rs
Anything testable appears to be low priority.

### cartesian_py.rs
Anything testable appears to be low priority.

### cartesian.rs
- `pub fn new(x_km: f64, y_km: f64, z_km: f64, vx_km_s: f64, vy_km_s: f64, vz_km_s: f64, epoch: Epoch, frame: Frame) -> Self`
- `pub fn distance_to_km(&self, other: &Self) -> PhysicsResult<f64>`
- `pub fn eq_within(&self, other: &Self, radial_tol_km: f64, velocity_tol_km_s: f64) -> bool`
- `pub fn abs_pos_diff_km(&self, other: &Self) -> PhysicsResult<f64>`
- `pub fn abs_vel_diff_km_s(&self, other: &Self) -> PhysicsResult<f64>`
- `pub fn rel_pos_diff(&self, other: &Self) -> PhysicsResult<f64>`
- `pub fn rel_vel_diff(&self, other: &Self) -> PhysicsResult<f64>`

### units.rs
Anything testable appears to be low priority.

## anise/src/orientations
### paths.rs
- `pub fn try_find_orientation_root(&self) -> Result<NaifId, OrientationError>`
- `pub fn orientation_path_to_root(&self, source: Frame, epoch: Epoch) -> Result<(usize, [Option<NaifId>; MAX_TREE_DEPTH]), OrientationError>`
- `pub fn common_orientation_path`

### rotate_to_parent.rs
- `pub fn rotation_to_parent(&self, source: Frame, epoch: Epoch) -> Result<DCM, OrientationError>`

### rotation.rs
- `rotate(&self, from_frame: Frame, to_frame: Frame, epoch: Epoch) -> Result<DCM, OrientationError>`
- `pub fn rotate_state_to(&self, position: Vector3, velocity: Vector3, from_frame: Frame, to_frame: Frame, epoch: Epoch, distance_unit: LengthUnit, time_unit: TimeUnit) -> Result<CartesianState, OrientationError>`


## anise/src/ephemerides
### translate_to_parent.rs
- `pub fn translate_to_parent(&self, source: Frame, epoch: Epoch, ) -> Result<CartesianState, EphemerisError>`

### translations.rs
- `pub fn translate(&self,target_frame: Frame,mut observer_frame: Frame,epoch: Epoch,ab_corr: Option<Aberration>,) -> Result<CartesianState, EphemerisError>`
- `pub fn translate_geometric(&self,target_frame: Frame,observer_frame: Frame,epoch: Epoch,) -> Result<CartesianState, EphemerisError>`
- `pub fn translate_to(&self,state: CartesianState,mut observer_frame: Frame,ab_corr: Option<Aberration>,) -> Result<CartesianState, EphemerisError>`
- `pub fn translate_state_to(&self,position: Vector3,velocity: Vector3,from_frame: Frame,observer_frame: Frame,epoch: Epoch,ab_corr: Option<Aberration>,distance_unit: LengthUnit,time_unit: TimeUnit,) -> Result<CartesianState, EphemerisError>`

### paths.rs
- `try_find_ephemeris_root(&self) -> Result<NaifId, EphemerisError>`
- `pub fn ephemeris_path_to_root(&self,source: Frame,epoch: Epoch,) -> Result<(usize, [Option<NaifId>; MAX_TREE_DEPTH]), EphemerisError>`
- `pub fn common_ephemeris_path(&self,from_frame: Frame,to_frame: Frame,epoch: Epoch,) -> Result<(usize, [Option<NaifId>; MAX_TREE_DEPTH], NaifId), EphemerisError>`

### mod.rs
Anything testable appears to be low priority.

## anise/src/astro
### aberration.rs
- `pub fn new(flag: &str) -> PhysicsResult<Option<Self>>`
- `stellar_aberration(target_pos_km: Vector3, obs_wrt_ssb_vel_km_s: Vector3, ab_corr: Aberration) -> PhysicsResult<Vector3>`

### mod.rs
- `pub fn is_valid(&self) -> bool`
- `pub fn py_new(epoch: Epoch,azimuth_deg: f64,elevation_deg: f64,range_km: f64,range_rate_km_s: f64,obstructed_by: Option<Frame>,) -> Self`
- `fn set_range_km(&mut self, range_km: f64) -> PyResult<()>`

### occultation.rs
- `pub fn factor(&self) -> f64`
- `pub fn is_visible(&self) -> bool`
- `pub fn is_obstructed(&self) -> bool`
- `pub fn is_partial(&self) -> bool`

### orbit.rs
- `pub fn try_keplerian(sma_km: f64,ecc: f64,inc_deg: f64,raan_deg: f64,aop_deg: f64,ta_deg: f64,epoch: Epoch,frame: Frame,) -> PhysicsResult<Self>`
- `pub fn try_keplerian_apsis_radii(r_a_km: f64,r_p_km: f64,inc_deg: f64,raan_deg: f64,aop_deg: f64,ta_deg: f64,epoch: Epoch,frame: Frame,) -> PhysicsResult<Self>`
- `pub fn try_keplerian_mean_anomaly(sma_km: f64,ecc: f64,inc_deg: f64,raan_deg: f64,aop_deg: f64,ma_deg: f64,epoch: Epoch,frame: Frame,) -> - PhysicsResult<Self>`
- `pub fn hvec(&self) -> PhysicsResult<Vector3>`
- `pub fn evec(&self) -> Result<Vector3, PhysicsError>`
- `pub fn ta_deg(&self) -> PhysicsResult<f64>`
- `pub fn energy_km2_s2(&self) -> PhysicsResult<f64>`
- `pub fn sma_km(&self) -> PhysicsResult<f64>`
- `pub fn ecc(&self) -> PhysicsResult<f64>`
- `pub fn period(&self) -> PhysicsResult<Duration>`
- `pub fn set_sma_km(&mut self, new_sma_km: f64) -> PhysicsResult<()>`
- `pub fn set_ecc(&mut self, new_ecc: f64) -> PhysicsResult<()>`
- `pub fn set_inc_deg(&mut self, new_inc_deg: f64) -> PhysicsResult<()>`

### orbit_geodedic.rs
- `pub fn try_keplerian_altitude(sma_altitude_km: f64,ecc: f64,inc_deg: f64,raan_deg: f64,aop_deg: f64,ta_deg: f64,epoch: Epoch,frame: Frame,) -> PhysicsResult<Self>`
- `pub fn try_keplerian_apsis_altitude(apo_alt_km: f64,peri_alt_km: f64,inc_deg: f64,raan_deg: f64,aop_deg: f64,ta_deg: f64,epoch: Epoch,frame: Frame,) -> PhysicsResult<Self>`
- `pub fn try_latlongalt(latitude_deg: f64,longitude_deg: f64,height_km: f64,angular_velocity_deg_s: f64,epoch: Epoch,frame: Frame,) -> PhysicsResult<Self>`
- `pub fn latlongalt(&self) -> PhysicsResult<(f64, f64, f64)>`

### utils.rs
- `pub fn compute_mean_to_true_anomaly_rad(ma_radians: f64, ecc: f64) -> PhysicsResult<f64>`

## anise/src/almanac