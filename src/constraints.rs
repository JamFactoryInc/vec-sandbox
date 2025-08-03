/// A const index used in compile-time bounds checking
/// 
/// supports reverse indices by setting the `REVERSE` flag
///
/// Reverse indices would be indices like -1, representing the index `length - 1`, or the last element in the collection.
pub struct ConstIndex<const I: usize, const REVERSE: bool>;

/// A wrapper around a constant usize value, for use with compile-time validation
pub struct ConstNum<const N: usize>;

/// A constraint 
pub trait NonZero<const TRUE: bool> {}

impl<const N: usize> NonZero<{ N > 0}> for ConstNum<N> {}

pub trait WithinBounds<const TRUE: bool> {}

// implement truthiness of `WithinBounds` for positive indices
impl<const INDEX: usize, const LENGTH: usize> WithinBounds<{ within_bounds(0, INDEX, LENGTH) }> for (
    ConstIndex<INDEX, false>,
    ConstNum<LENGTH>
) {}

// implement truthiness of `WithinBounds` for negative indices
impl<const NEG_INDEX: usize, const LENGTH: usize> WithinBounds<{ within_bounds(1, NEG_INDEX, LENGTH + 1) }> for (
    ConstIndex<NEG_INDEX, true>,
    ConstNum<LENGTH>
) {}

/// a const fn used in type logic to evaluate whether a given constant `value` is within the bounds defined by `min` and `max_exclusive`
/// 
/// currently used for compile-time bounds checking
const fn within_bounds(min: usize, value: usize, max_exclusive: usize) -> bool {
    value >= min && value < max_exclusive
}