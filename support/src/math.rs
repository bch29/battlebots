pub mod vector {
    use cgmath;
    use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign};

    #[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Default, Serialize, Deserialize)]
    pub struct Vector2 {
        pub x: f64,
        pub y: f64,
    }

    impl Vector2 {
        pub fn new(x: f64, y: f64) -> Self {
            Vector2 { x: x, y: y }
        }

        pub fn zero() -> Self {
            Vector2::new(0.0, 0.0)
        }

        pub fn from_cgmath(v: cgmath::Vector2<f64>) -> Self {
            Vector2::new(v.x, v.y)
        }

        pub fn into_cgmath(self) -> cgmath::Vector2<f64> {
            cgmath::Vector2::new(self.x, self.y)
        }
    }

    impl<'a, 'b> Add<&'a Vector2> for &'b Vector2 {
        type Output = Vector2;

        fn add(self, other: &Vector2) -> Vector2 {
            Vector2::new(self.x + other.x, self.y + other.y)
        }
    }

    impl Add<Vector2> for Vector2 {
        type Output = Vector2;

        fn add(self, other: Self) -> Self {
            &self + &other
        }
    }

    impl<'a> AddAssign<&'a Vector2> for Vector2 {
        fn add_assign(&mut self, other: &Vector2) {
            *self = (self as &Vector2) + other
        }
    }

    impl AddAssign<Vector2> for Vector2 {
        fn add_assign(&mut self, other: Vector2) {
            *self = (self as &Vector2) + &other
        }
    }

    impl<'a, 'b> Sub<&'a Vector2> for &'b Vector2 {
        type Output = Vector2;

        fn sub(self, other: &Vector2) -> Vector2 {
            Vector2::new(self.x - other.x, self.y - other.y)
        }
    }

    impl Sub<Vector2> for Vector2 {
        type Output = Vector2;

        fn sub(self, other: Self) -> Self {
            &self - &other
        }
    }

    impl<'a> SubAssign<&'a Vector2> for Vector2 {
        fn sub_assign(&mut self, other: &Vector2) {
            *self = (self as &Vector2) - other
        }
    }

    impl SubAssign<Vector2> for Vector2 {
        fn sub_assign(&mut self, other: Vector2) {
            *self = (self as &Vector2) - &other
        }
    }

    impl<'a> Mul<f64> for &'a Vector2 {
        type Output = Vector2;

        fn mul(self, scalar: f64) -> Vector2 {
            Vector2::new(self.x * scalar, self.y * scalar)
        }
    }

    impl Mul<f64> for Vector2 {
        type Output = Vector2;

        fn mul(self, scalar: f64) -> Vector2 {
            &self * scalar
        }
    }

    impl MulAssign<f64> for Vector2 {
        fn mul_assign(&mut self, scalar: f64) {
            *self = (self as &Vector2) * scalar
        }
    }
}

pub use self::vector::*;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Clamped<T> {
    min: T,
    max: T,
}

impl<T: PartialOrd + Clone> Clamped<T> {
    pub fn new(min: T, max: T) -> Self {
        Clamped {
            min: min,
            max: max
        }
    }

    pub fn clamp(&self, val: T) -> T {
        if val < self.min {
            self.min.clone()
        } else if val > self.max {
            self.max.clone()
        } else {
            val
        }
    }

    pub fn check(&self, val: T) -> Result<T, T> {
        if self.min <= val && val <= self.max {
            Ok(val)
        } else {
            Err(val)
        }
    }
}
