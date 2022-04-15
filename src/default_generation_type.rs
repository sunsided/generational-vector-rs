use num_traits::One;
use std::num::NonZeroUsize;
use std::ops::{Add, AddAssign, Deref, Mul};

/// The default generation type.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DefaultGenerationType(NonZeroUsize);

impl Default for DefaultGenerationType {
    fn default() -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

impl Deref for DefaultGenerationType {
    type Target = NonZeroUsize;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Mul for DefaultGenerationType {
    type Output = DefaultGenerationType;

    fn mul(self, _rhs: Self) -> Self::Output {
        // required for Add()
        unimplemented!()
    }
}

impl One for DefaultGenerationType {
    #[inline]
    fn one() -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

impl Add for DefaultGenerationType {
    type Output = DefaultGenerationType;

    fn add(self, rhs: Self) -> Self::Output {
        let value = self.0.get() + rhs.0.get();
        if value == 0 {
            panic!("overflow of generation value")
        }
        Self(unsafe { NonZeroUsize::new_unchecked(value) })
    }
}

impl AddAssign for DefaultGenerationType {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (*self + rhs).0;
    }
}
