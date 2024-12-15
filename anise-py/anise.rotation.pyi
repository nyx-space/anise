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