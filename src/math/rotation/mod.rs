use nalgebra::Vector3;

pub trait Rotation<T> {
    // What do we want as a common interface for rotations?

    // My suggestion :
    fn new(axis: Vector3<T>, angle: T) -> Self;
    fn axis(&self) -> Vector3<T>;
    fn angle(&self) -> T; // Rotation angle in radians around the rotation axis

    //fn compose<T2, Rot2<T2> where Rot2<T2> : !ComposeWith<Rot<T>>>(&self, other: &Rot2<T>) -> (Vector3<T>, T) {
    // Composition of two rotations with possibly differing precision

    // Imo the best approach would be to have two implementations
    // A specific one, for example composition of two quaternions is a multiplication that can be made very efficient
    // And a generic one that would be the conversion into axis/angle of both rotations and then compose the axis/angle representations
    // This could probably be implemented with smart usage of a ComposeWith trait

    // Reference for the generic approach : https://math.stackexchange.com/questions/382760/composition-of-two-axis-angle-rotations
    // We should probably get the formulas through Herbie floating point optimization

    // let alpha = self.angle(); let beta = other.angle(); let l = self.axis(); let m = other.axis();
    // let new_angle = 2*acos(cos(alpha/2)  * cos(beta/2) - l.dot(m) * sin(alpha/2) * sin(beta/2));
    // let new_axis = (2/new_angle)*asin( sin(alpha/2) * cos(beta/2) * l + cos(alpha/2) * sin(beta/2) * m + sin(alpha/2) * sin(beta/2) * l.cross(m) );

    // Will automatically be coerced into a RawRotation via the From trait, if needed
    // (new_axis, new_angle)
    // }

    // fn inverse(&self) -> Self ?
}

pub trait ComposeWith<T> {
    type Output;

    fn compose(&self, other: &T) -> Self::Output;
}

// Possible tests for all rotations :
// Check transformation is correct : rotation -> RawRotation -> rotation
// Check composition rules are correct with inverse rotation ?
// Check edge cases : all-zero axis, non-unit axis, zero angle, , NaN/Inf angle, angle > PI or < PI, others ?
// Check against other libraries ?
// Check performance differences ?

/// Raw rotation, represented as an axis and angle
pub struct RawRotation<T> {
    pub axis: Vector3<T>,
    pub angle: T,
}

impl<T: Clone> Rotation<T> for RawRotation<T> {
    fn new(axis: Vector3<T>, angle: T) -> Self {
        Self { axis, angle }
    }

    fn axis(&self) -> Vector3<T> {
        self.axis.clone()
    }

    fn angle(&self) -> T {
        self.angle.clone()
    }
}

impl<T: Clone> From<(Vector3<T>, T)> for RawRotation<T> {
    fn from((axis, angle): (Vector3<T>, T)) -> Self {
        Self::new(axis, angle)
    }
}

/// Unit quaternion with a specified tolerance for normalization
pub struct RotQuaternion<T> {
    pub w: T,
    pub x: T,
    pub y: T,
    pub z: T,

    /// Tolerances are used in the normalization process to make sure
    /// that the quaternion is normalized with the needed guarantees
    // Should tolerances be optional ? I'd say they should be mandatory but have a fair default value
    pub abs_tolerance: T,
    pub rel_tolerance: T,
    //// Source and destination reference frames are used for runtime check of
    //// the correctness of the rotation
    // TODO : we should have a way to disable runtime checks
    // pub source : RefFrameHash,
    // pub destination : RefFrameHash
}

impl<T: core::convert::From<f32>> RotQuaternion<T> {
    pub fn new(w: T, x: T, y: T, z: T) -> Self {
        // TODO : Check the quaternion is unitary with the correct tolerance
        todo!();

        Self {
            w,
            x,
            y,
            z,
            abs_tolerance: T::from(1e-6_f32),
            rel_tolerance: T::from(1e-6_f32),
        }
    }
}

impl<T> RotQuaternion<T> {
    pub fn new_with_tol(w: T, x: T, y: T, z: T, abs_tol: T, rel_tol: T) -> Self {
        // TODO : Check the quaternion is unit with the correct tolerance
        todo!();

        Self {
            w,
            x,
            y,
            z,
            abs_tolerance: abs_tol,
            rel_tolerance: rel_tol,
        }
    }
}

impl<T> Rotation<T> for RotQuaternion<T> {
    fn new(axis: Vector3<T>, angle: T) -> Self {
        todo!();
    }
    fn axis(&self) -> Vector3<T> {
        todo!()
    }

    fn angle(&self) -> T {
        todo!()
    }
}

impl<T> ComposeWith<RotQuaternion<T>> for RotQuaternion<T> {
    // TODO : allow differing precisions ?
    type Output = Self;

    fn compose(&self, other: &Self) -> Self::Output {
        todo!()
    }
}

/// Modified Rodrigues parameter
pub struct MRP {}

// etc... :)
// Good night !
