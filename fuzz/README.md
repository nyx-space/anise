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
