mod default_generation_type;
pub mod iterators;
pub mod vector;

pub use default_generation_type::DefaultGenerationType;
use num_traits::One;
use std::ops::Add;
pub use vector::DeletionResult;

/// Type alias to simplify construction of generational vectors.
pub type GenerationalVector<T> = vector::GenerationalVector<T, DefaultGenerationType>;

/// Alias for required traits on the type used for the generation value.
pub trait GenerationType: One + Copy + Add<Output = Self> + PartialEq {}

/// Automatic implementation of `GenerationType` for all matching types.
impl<T> GenerationType for T where T: One + Copy + Add<Output = T> + PartialEq {}
