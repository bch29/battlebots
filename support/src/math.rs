/// Serialisable 2D vectors.
pub mod vector {
    use cgmath;
    use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign};

    /// A two dimensional vector with `f64` components.
    #[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Default, Serialize, Deserialize)]
    pub struct Vector2 {
        pub x: f64,
        pub y: f64,
    }

    impl Vector2 {
        /// Creates a new vector with the given `x` and `y` components.
        #[inline]
        pub fn new(x: f64, y: f64) -> Self {
            Vector2 { x: x, y: y }
        }

        /// Returns the zero vector.
        #[inline]
        pub fn zero() -> Self {
            Vector2::new(0.0, 0.0)
        }

        /// Converts a `Vector2` from `cgmath` into a serialisable vector.
        #[inline]
        pub fn from_cgmath(v: cgmath::Vector2<f64>) -> Self {
            Vector2::new(v.x, v.y)
        }

        /// Converts the vector into `cgmath`'s representation.
        #[inline]
        pub fn into_cgmath(self) -> cgmath::Vector2<f64> {
            cgmath::Vector2::new(self.x, self.y)
        }
    }

    impl<'a, 'b> Add<&'a Vector2> for &'b Vector2 {
        type Output = Vector2;

        #[inline]
        fn add(self, other: &Vector2) -> Vector2 {
            Vector2::new(self.x + other.x, self.y + other.y)
        }
    }

    impl Add<Vector2> for Vector2 {
        type Output = Vector2;

        #[inline]
        fn add(self, other: Self) -> Self {
            &self + &other
        }
    }

    impl<'a> AddAssign<&'a Vector2> for Vector2 {
        #[inline]
        fn add_assign(&mut self, other: &Vector2) {
            *self = (self as &Vector2) + other
        }
    }

    impl AddAssign<Vector2> for Vector2 {
        #[inline]
        fn add_assign(&mut self, other: Vector2) {
            *self = (self as &Vector2) + &other
        }
    }

    impl<'a, 'b> Sub<&'a Vector2> for &'b Vector2 {
        type Output = Vector2;

        #[inline]
        fn sub(self, other: &Vector2) -> Vector2 {
            Vector2::new(self.x - other.x, self.y - other.y)
        }
    }

    impl Sub<Vector2> for Vector2 {
        type Output = Vector2;

        #[inline]
        fn sub(self, other: Self) -> Self {
            &self - &other
        }
    }

    impl<'a> SubAssign<&'a Vector2> for Vector2 {
        #[inline]
        fn sub_assign(&mut self, other: &Vector2) {
            *self = (self as &Vector2) - other
        }
    }

    impl SubAssign<Vector2> for Vector2 {
        #[inline]
        fn sub_assign(&mut self, other: Vector2) {
            *self = (self as &Vector2) - &other
        }
    }

    impl<'a> Mul<f64> for &'a Vector2 {
        type Output = Vector2;

        #[inline]
        fn mul(self, scalar: f64) -> Vector2 {
            Vector2::new(self.x * scalar, self.y * scalar)
        }
    }

    impl Mul<f64> for Vector2 {
        type Output = Vector2;

        #[inline]
        fn mul(self, scalar: f64) -> Vector2 {
            &self * scalar
        }
    }

    impl MulAssign<f64> for Vector2 {
        #[inline]
        fn mul_assign(&mut self, scalar: f64) {
            *self = (self as &Vector2) * scalar
        }
    }
}

/// Serialisable clamped values.
pub mod clamped {

    /// Specifies a range of allowed values which can be clamped to or checked
    /// against.
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Clamped<T> {
        pub min: T,
        pub max: T,
    }

    impl<T: PartialOrd + Clone> Clamped<T> {

        /// Creates a new clamped value with the given min and max.
        #[inline]
        pub fn new(min: T, max: T) -> Self {
            Clamped {
                min: min,
                max: max
            }
        }

        /// Clamps the given value to between the `min` and `max` values.
        #[inline]
        pub fn clamp(&self, val: T) -> T {
            if val < self.min {
                self.min.clone()
            } else if val > self.max {
                self.max.clone()
            } else {
                val
            }
        }

        /// Checks if `val` is within `min` and `max` (inclusive). If so,
        /// returns `Ok(val)`, otherwise returns `Err(val)`.
        #[inline]
        pub fn check(&self, val: T) -> Result<T, T> {
            if self.min <= val && val <= self.max {
                Ok(val)
            } else {
                Err(val)
            }
        }
    }
}

pub use self::vector::*;
pub use self::clamped::*;
