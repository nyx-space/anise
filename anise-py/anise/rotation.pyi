import np
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
    def from_align_and_clock(primary_body_axis: np.array, primary_inertial_vec: np.array, secondary_body_axis: np.array, secondary_inertial_vec: np.array, from_id: int, to_id: int) -> DCM:
        """Constructs a DCM using the "Align and Clock" (Two-Vector Targeting / TRIAD) method.

This defines a rotation based on two geometric constraints:
1. **Align**: The `primary_body_axis` is aligned exactly with the `primary_inertial_vec`.
2. **Clock**: The `secondary_body_axis` is aligned as closely as possible with the `secondary_inertial_vec`.

This constructs the rotation matrix $R_{from \\to to}$.

# Arguments
* `primary_body_axis` - The axis in the "from" frame to align (e.g. Sensor Boresight).
* `primary_inertial_vec` - The target vector in the "to" frame (e.g. Vector to Earth).
* `secondary_body_axis` - The axis in the "from" frame to clock (e.g. Solar Panel Normal).
* `secondary_inertial_vec` - The target vector in the "to" frame (e.g. Vector to Sun).
* `from` - The ID of the source frame.
* `to` - The ID of the destination frame."""

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
    """Represents the orientation of a rigid body or a coordinate frame transformation in three-dimensional space using Euler parameters.

Euler parameters, also known as unit quaternions, are a set of four parameters `w`, `x`, `y`, `z`.
They are particularly useful because they avoid gimbal lock, are more compact than rotation matrices,
and allow for smooth interpolation (SLERP).

# Conventions

ANISE strictly adheres to the following conventions to ensure consistency with `DCM` and standard
Guidance, Navigation, and Control (GNC) mathematics:

1. **Hamiltonian Algebra:** The quaternion multiplication follows the right-handed rule where \\
`i * j = k`. This is the standard in robotics and computer graphics, but distinct from the
"JPL/Shuster" convention (`i * j = -k`) sometimes found in legacy aerospace software.

2. **Passive Rotation (Coordinate Transformation):** A quaternion defined with `from: A` and `to: B`
represents the transformation of **coordinates** from Frame A to Frame B.
* The rotation of a vector `v` is computed as: `v_B = q.conjugate() * v_A * q`.
* This matches the behavior of a Direction Cosine Matrix (DCM): `v_B = [DCM_A->B] * v_A`.

3. **Operator Composition (Backward Chaining):** Rotations are composed in "Operator Order",
matching matrix multiplication rules.
* To compute the rotation A -> C, you multiply B->C by A->B.
* `q_A_to_C = q_B_to_C * q_A_to_B`
* ANISE enforces strict frame checking: `LHS.from` must equal `RHS.to`.

# Definitions

Euler parameters are defined in terms of the principal rotation vector. If a frame is rotated
by an angle `θ` about a unit axis `e = [e1, e2, e3]`:

* `w = cos(θ / 2)`
* `x = e1 * sin(θ / 2)`
* `y = e2 * sin(θ / 2)`
* `z = e3 * sin(θ / 2)`

These parameters satisfy `w^2 + x^2 + y^2 + z^2 = 1`, which means they represent
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
        """Represents the orientation of a rigid body or a coordinate frame transformation in three-dimensional space using Euler parameters.

Euler parameters, also known as unit quaternions, are a set of four parameters `w`, `x`, `y`, `z`.
They are particularly useful because they avoid gimbal lock, are more compact than rotation matrices,
and allow for smooth interpolation (SLERP).

# Conventions

ANISE strictly adheres to the following conventions to ensure consistency with `DCM` and standard
Guidance, Navigation, and Control (GNC) mathematics:

1. **Hamiltonian Algebra:** The quaternion multiplication follows the right-handed rule where \\
`i * j = k`. This is the standard in robotics and computer graphics, but distinct from the
"JPL/Shuster" convention (`i * j = -k`) sometimes found in legacy aerospace software.

2. **Passive Rotation (Coordinate Transformation):** A quaternion defined with `from: A` and `to: B`
represents the transformation of **coordinates** from Frame A to Frame B.
* The rotation of a vector `v` is computed as: `v_B = q.conjugate() * v_A * q`.
* This matches the behavior of a Direction Cosine Matrix (DCM): `v_B = [DCM_A->B] * v_A`.

3. **Operator Composition (Backward Chaining):** Rotations are composed in "Operator Order",
matching matrix multiplication rules.
* To compute the rotation A -> C, you multiply B->C by A->B.
* `q_A_to_C = q_B_to_C * q_A_to_B`
* ANISE enforces strict frame checking: `LHS.from` must equal `RHS.to`.

# Definitions

Euler parameters are defined in terms of the principal rotation vector. If a frame is rotated
by an angle `θ` about a unit axis `e = [e1, e2, e3]`:

* `w = cos(θ / 2)`
* `x = e1 * sin(θ / 2)`
* `y = e2 * sin(θ / 2)`
* `z = e3 * sin(θ / 2)`

These parameters satisfy `w^2 + x^2 + y^2 + z^2 = 1`, which means they represent
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