#![feature(generic_const_exprs)]

use std::hint;
use constraints::{NonZero, ConstIndex, WithinBounds, ConstNum};

mod use_cases;
mod constraints;

/// a wrapper around a mutable vector reference, allowing for compile-time bounds checks
/// 
/// `'vec`: The lifetime of the wrapped vector. This is used as normal with reference fields, 
/// but also enables us to explicitly return references to vector elements that outlive the sandboxed scope
/// 
/// `MIN_LEN`: The minimum length that this vector is guaranteed to be at compile-time.
/// The vector may be longer than this, but this allows guaranteed access to up to `MIN_LEN` elements without checking at runtime.
///
/// For example, if `MIN_LEN` is 2, we can safely retrieve the `first()` and `last()` element. We can also safely `pop()`, and `get()` indices 0, 1, -1, and -2.
pub struct SandboxMut<'vec, const MIN_LEN: usize, T> {
    vec: &'vec mut Vec<T>,
}

impl<'vec, const MIN_LEN: usize, T> SandboxMut<'vec, MIN_LEN, T> {

    /// returns a reference to a constant index that is assumed to be valid
    /// 
    /// Safety: as long as the provided `INDEX` is guaranteed to be valid at compile time, this function is safe to use
    unsafe fn get_checked_index<const INDEX: isize>(vec: &Vec<T>) -> &T {
        if INDEX < 0 {
           vec.get_unchecked(vec.len() - INDEX.unsigned_abs())
        } else {
            vec.get_unchecked(INDEX as usize)
        }
    }

    /// returns a mutable reference to a constant index that is assumed to be valid
    ///
    /// Safety: as long as the provided `INDEX` is guaranteed to be valid at compile time, this function is safe to use
    unsafe fn get_mut_checked_index<const INDEX: isize>(vec: &mut Vec<T>) -> &mut T {
        if INDEX < 0 {
            let len = vec.len();
            vec.get_unchecked_mut(len - INDEX.unsigned_abs())
        } else {
            vec.get_unchecked_mut(INDEX as usize)
        }
    }
    
    /// accepts a positive or negative constant index and returns a reference to the element at that index.
    /// 
    /// bounds checks are performed at compile time to ensure the index is valid
    /// 
    /// Usage:
    /// ```
    /// use vec_sandbox::Sandboxed;
    /// 
    /// let mut vec: Vec<usize> = vec![1];
    /// let sandbox = vec.sandboxed()
    ///     .push(2);
    /// 
    /// let first_val: &usize = sandbox.get::<0>();
    /// println!("First value: {}", first_val);
    /// 
    /// let last_val: &usize = sandbox.get::<-1>();
    /// println!("Last value: {}", last_val);
    /// ```
    pub fn get<const INDEX: isize>(&self) -> &T where
        (ConstIndex<{ INDEX.unsigned_abs() }, { INDEX < 0 }>, ConstNum<MIN_LEN>): WithinBounds<true>
    {
        unsafe {
            Self::get_checked_index::<INDEX>(self.vec)
        }
    }

    /// takes ownership of the sandbox and returns a reference to the element in the underlying vector,
    /// which outlives the sandbox instance and can be safely returned from an enclosing function
    /// 
    /// otherwise performs the same action as `get()`, accepting a positive or negative constant index
    ///
    /// Usage:
    /// ```
    /// use vec_sandbox::Sandboxed;
    ///
    /// fn push_and_get_reference(vec: &mut Vec<usize>) -> &usize {
    ///     vec.sandboxed()
    ///         .push(2)
    ///         .release_get::<-1>() // .get::<-1>() returns a reference bound to the lifetime of the sandbox, not the backing vector
    /// }
    /// ```
    pub fn release_get<const INDEX: isize>(self) -> &'vec T where
        (ConstIndex<{ INDEX.unsigned_abs() }, { INDEX < 0 }>, ConstNum<MIN_LEN>): WithinBounds<true>
    {
        unsafe {
            Self::get_checked_index::<INDEX>(self.vec)
        }
    }

    pub fn swap<const LEFT_INDEX: isize, const RIGHT_INDEX: isize>(&mut self) where
        (ConstIndex<{ LEFT_INDEX.unsigned_abs() }, { LEFT_INDEX < 0 }>, ConstNum<MIN_LEN>): WithinBounds<true>,
        (ConstIndex<{ RIGHT_INDEX.unsigned_abs() }, { RIGHT_INDEX < 0 }>, ConstNum<MIN_LEN>): WithinBounds<true>
    {
        self.vec.swap(LEFT_INDEX as usize, RIGHT_INDEX as usize)
    }

    pub fn release_get_mut<const INDEX: isize>(self) -> &'vec mut T where (
        ConstIndex<{ INDEX.unsigned_abs() }, { INDEX < 0 }>,
        ConstNum<MIN_LEN>
    ): WithinBounds<true> {
        unsafe {
            Self::get_mut_checked_index::<INDEX>(self.vec)
        }
    }

    /// takes ownership of this sandbox, pushes the value to the underlying vector, 
    /// and returns a new sandbox with an incremented guaranteed length
    /// 
    /// This pattern is most conducive to chained method calls like 
    /// ```
    /// use vec_sandbox::Sandboxed;
    /// 
    /// let mut vec = vec![];
    /// vec.sandboxed().push(1).push(2)
    /// ```
    /// but can also take advantage of variable shadowing / re-definition
    /// ```
    /// use vec_sandbox::Sandboxed;
    /// 
    /// let mut vec = vec![];
    /// let sandbox = vec.sandboxed();
    /// let sandbox = sandbox.push(1);
    /// let sandbox = sandbox.push(2);
    /// ```
    pub fn push(self, value: T) -> SandboxMut<'vec, { MIN_LEN + 1 }, T> {
        self.vec.push(value);
        SandboxMut {
            vec: self.vec,
        }
    }
    
    /// Performs a runtime check to provide compile-time guarantees about the `MIN_LEN` of a vector in the resulting `Ok` path.
    /// 
    /// Usage:
    /// ```
    /// use vec_sandbox::Sandboxed;
    ///
    /// let mut vec = vec!["one", "two", "three"];
    ///
    /// // the guaranteed length of the sandbox is 1,
    /// // since we've performed one sandboxed `push` operation
    /// let sandboxed = vec.sandboxed().push("four"); 
    ///
    /// match sandboxed.try_guarantee_length::<3>() {
    ///     // case if the length of the sandbox is at least 3 elements
    ///     // the sandbox instance bound here has a guaranteed length of 3, so we can safely use indices -3 ..= 2
    ///     Ok(at_least_3) => {
    ///         println!("{}", at_least_3.get::<2>()); // safely access the third element (index 2)
    ///         println!("{}", at_least_3.get::<-3>()); // safely access the third-to-last element (index -3 i.e. `len() - 3`)
    ///     }
    ///     // case if the length of the sandbox is less than 3 elements
    ///     // the sandbox instance bound here inherits the original guaranteed length of 1
    ///     Err(default) => {
    ///         println!("vec is less than 3, but at least 1");
    ///         println!("{}", default.get::<0>()); // safely access the first
    ///         println!("{}", default.get::<-1>()); // safely access the last element
    ///     }
    /// }
    /// ```
    pub fn try_guarantee_length<const LEN: usize>(self) -> Result<SandboxMut<'vec, LEN, T>, Self> {
        unsafe {
            hint::assert_unchecked(self.vec.len() >= MIN_LEN);
        }
        match self.vec.len() >= LEN {
            true => Ok(SandboxMut {
                vec: self.vec,
            }),
            _ => Err(self)
        }
    }
}

/// A collection of vector methods that are valid as long as the vector has at least one element
/// i.e. if the vector is non-empty, as the name would suggest.
/// 
/// Most methods that apply here are index-agnostic operations like first, last, and pop, 
/// but there are likely other exploitations of the non-empty guarantee that we can explore
pub trait NonEmptyOps<'vec, const MIN_LEN: usize, T> {
    /// retrieves a reference to the first value in the vector
    fn first(&'vec self) -> &'vec T;

    /// retrieves a reference to the last value in the vector
    fn last(&self) -> &T;

    /// removes the last value in the vector and a tuple containing the new sandbox and the removed value
    /// 
    /// Usage:
    /// ```
    /// let mut vec = vec![];
    /// let sandboxed = vec.sandboxed().push(1);
    /// let (sandboxed, removed_value) = sandboxed.pop();
    /// ```
    fn pop(self) -> (SandboxMut<'vec, { MIN_LEN - 1 }, T>, T);
}

impl<'vec, const MIN_LEN: usize, T> NonEmptyOps<'vec, MIN_LEN, T> for SandboxMut<'vec, MIN_LEN, T>
where ConstNum<MIN_LEN>: NonZero<true> {
    fn first(&'vec self) -> &'vec T {
        self.vec.first().unwrap()
    }

    fn last(&self) -> &T {
        let len = self.vec.len();
        self.vec.get(len - 1).unwrap()
    }

    fn pop(self) -> (SandboxMut<'vec, {MIN_LEN - 1}, T>, T) {
        let popped = self.vec.pop().unwrap();
        (
            SandboxMut { vec: self.vec },
            popped
        )
    }
}

pub trait GuaranteedLength<'vec, T> where Self: 'vec {
    /// Equivalent to `with_min_length::<1>()`
    /// 
    /// If valid at runtime, returns a Sandbox exposing operations available for non-empty collections
    fn as_non_empty(&'vec mut self) -> Option<SandboxMut<'vec, 1, T>>;
    
    /// Checks the length of the collection at runtime
    /// 
    /// If the collection length is at least the indicated `LEN`, returns a Sandbox with `MIN_LEN` = `LEN`
    fn with_min_length<const LEN: usize>(&'vec mut self) -> Option<SandboxMut<'vec, LEN, T>>;
}

impl<'vec, T, const MIN_LEN: usize> GuaranteedLength<'vec, T> for SandboxMut<'vec, MIN_LEN, T> {
    fn as_non_empty(&'vec mut self) -> Option<SandboxMut<'vec, 1, T>> {
        Some(SandboxMut {
            vec: self.vec,
        }).filter(move |sandbox| sandbox.vec.len() >= 1)
    }
    
    fn with_min_length<const LEN: usize>(&'vec mut self) -> Option<SandboxMut<'vec, LEN, T>> {
        unsafe {
            hint::assert_unchecked(self.vec.len() >= MIN_LEN);
        }
        Some(SandboxMut {
            vec: self.vec,
        }).filter(move |sandbox| sandbox.vec.len() >= LEN)
    }
}

impl<'vec, T> GuaranteedLength<'vec, T> for Vec<T> where Self: 'vec {
    fn as_non_empty(&'vec mut self) -> Option<SandboxMut<'vec, 1, T>> {
        Some(SandboxMut {
            vec: self,
        }).filter(move |sandbox| sandbox.vec.len() >= 1)
    }
    
    fn with_min_length<const LEN: usize>(&'vec mut self) -> Option<SandboxMut<'vec, LEN, T>> {
        Some(SandboxMut {
            vec: self,
        }).filter(move |sandbox| sandbox.vec.len() >= LEN)
    }
}

/// A trait used to add the sandbox methods to `Vec`
pub trait Sandboxed<'vec, T> where T: 'vec {
    /// gets a `SandboxMut` instance to allow for compile-time-checked operations on the given vector
    /// 
    /// Usage:
    /// ```
    /// use vec_sandbox::Sandboxed;
    ///
    /// let mut vec = vec!["first"];
    /// let sandbox = vec.sandboxed();
    /// let tail = sandbox.push("last").release_get::<-1>();
    /// println!("{}", tail) // outputs "last"
    /// ```
    fn sandboxed(&'vec mut self) -> SandboxMut<'vec, 0, T>;
    /// runs the given closure with a `SandboxMut` instance referencing the given vector, allowing for compile-time-checked operations
    /// 
    /// Usage:
    /// ```
    /// use vec_sandbox::Sandboxed;
    ///
    /// let mut vec = vec!["first"];
    /// let tail = vec.sandboxed_scope(|sandbox| {
    ///     sandbox.push("last").release_get::<-1>()
    /// });
    /// println!("{}", tail) // outputs "last"
    /// ```
    fn sandboxed_scope<R, F: FnOnce(SandboxMut<'vec, 0, T>) -> R>(&'vec mut self, sandboxed_fn: F) -> R;

    
}

impl<'vec, T> Sandboxed<'vec, T> for Vec<T> where T: 'vec {
    fn sandboxed(&'vec mut self) -> SandboxMut<'vec, 0, T> {
        SandboxMut { vec: self }
    }

    fn sandboxed_scope<R, F: FnOnce(SandboxMut<'vec, 0, T>) -> R>(&'vec mut self, sandboxed_fn: F) -> R {
        let sandbox = SandboxMut { vec: self };
        sandboxed_fn(sandbox)
    }
}