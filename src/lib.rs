#![feature(generic_const_exprs)]
#![feature(core_intrinsics)]

mod examples;

/// a representation of a compile-time index
/// 
/// supports the concept of reverse indices -- e.g. -1 representing the last element -- by setting the `REVERSE` flag
pub struct SafeIndex<const I: usize, const REVERSE: bool>;

/// a wrapper around a mutable vector reference, allowing for compile-time bounds checks
pub struct SandboxMut<'vec, const MIN_LEN: usize, T> {
    vec: &'vec mut Vec<T>,
}

impl<'vec, const MIN_LEN: usize, T> SandboxMut<'vec, MIN_LEN, T> {

    /// accepts a positive or negative constant index and returns a reference to the element at that index.
    /// 
    /// bounds checks are performed at compile time to ensure the index is valid
    /// 
    /// Example:
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
    pub fn get<const INDEX: isize>(&self) -> &T where (
        SafeIndex<{ INDEX.unsigned_abs() }, { INDEX < 0 }>,
        _Num<MIN_LEN>
    ): WithinBounds<true> {
        let index_usize: usize = INDEX.unsigned_abs();
        if INDEX < 0 {
            self.vec.get(self.vec.len() - index_usize).unwrap()
        } else {
            self.vec.get(index_usize).unwrap()
        }
    }
    
    /// takes ownership of the sandbox and returns a reference to the element in the underlying vector,
    /// which may be safely returned from the sandbox's enclosing function
    /// 
    /// otherwise performs the same as `get()`, accepting a positive or negative constant index
    ///
    /// Example:
    /// ```
    /// use vec_sandbox::Sandboxed;
    ///
    /// fn push_and_get_reference(vec: &mut Vec<usize>) -> &usize {
    ///     vec.sandboxed()
    ///         .push(2)
    ///         .return_get::<-1>() // .get::<-1>() returns a reference bound to the lifetime of the sandbox, not the backing vector
    /// }
    /// ```
    pub fn return_get<const INDEX: isize>(self) -> &'vec T where (
             SafeIndex<{ INDEX.unsigned_abs() }, { INDEX < 0 }>,
             _Num<MIN_LEN>
         ): WithinBounds<true> {
        let index_usize: usize = INDEX.unsigned_abs();
        if INDEX < 0 {
            self.vec.get(self.vec.len() - index_usize).unwrap()
        } else {
            self.vec.get(index_usize).unwrap()
        }
    }

    /// takes ownership of this sandbox, pushes the value to the underlying vector, 
    /// and returns a new sandbox with an incremented guaranteed length
    /// 
    /// Most conducive to chained method calls
    /// ```
    /// vec![].sandboxed().push(1).push(2)
    /// ```
    /// but can also take advantage of variable shadowing / re-definition
    /// ```
    /// let sandbox = vec![].sandboxed();
    /// let sandbox = sandbox.push(1);
    /// let sandbox = sandbox.push(2);
    /// ```
    pub fn push(self, value: T) -> SandboxMut<'vec, { MIN_LEN + 1 }, T> {
        self.vec.push(value);
        SandboxMut {
            vec: self.vec,
        }
    }
    
    /// Example:
    /// 
    /// ```
    /// use vec_sandbox::Sandboxed;
    /// 
    /// let mut vec: Vec<&str> = vec!["one", "two", "three"];
    /// 
    /// // the guaranteed length of the sandbox is 1,
    /// // since we've performed one sandboxed `push` operation
    /// let sandboxed = vec.sandboxed().push("four"); 
    /// 
    /// match sandboxed.check_len::<3>() {
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
    pub fn check_len<const LEN: usize>(self) -> Result<SandboxMut<'vec, LEN, T>, Self> {
        unsafe {
            core::hint::assert_unchecked(self.vec.len() > MIN_LEN);
        }
        match self.vec.len() >= LEN {
            true => Ok(SandboxMut {
                vec: self.vec,
            }),
            _ => Err(self)
        }
    }
}

pub struct _Num<const N: usize>;

pub trait NonZero<const TRUE: bool> {}
impl<const N: usize> NonZero<{ N > 0}> for _Num<N> {}

const fn within_bounds(min: usize, max_exclusive: usize, index: usize) -> bool {
    index >= min && index < max_exclusive
}

pub trait WithinBounds<const TRUE: bool> {}

impl<const INDEX: usize, const LENGTH: usize> WithinBounds<{ within_bounds(0, LENGTH, INDEX) }> for (
    SafeIndex<INDEX, false>,
    _Num<LENGTH>
) {}

// idx -1, actual len 5
// 
impl<const NEG_INDEX: usize, const LENGTH: usize> WithinBounds<{ within_bounds(1, LENGTH + 1, NEG_INDEX) }> for (
    SafeIndex<NEG_INDEX, true>,
    _Num<LENGTH>
) {}

pub trait NonEmptyOps<'vec, const MIN_LEN: usize, T> {
    fn first(&'vec self) -> &'vec T;
    fn last(&self) -> &T;
    fn pop(self) -> (SandboxMut<'vec, { MIN_LEN - 1 }, T>, T);
}

impl<'vec, const MIN_LEN: usize, T> NonEmptyOps<'vec, MIN_LEN, T> for SandboxMut<'vec, MIN_LEN, T> where _Num<MIN_LEN>: NonZero<true> {
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

pub trait Sandboxed<'vec, T> where T: 'vec {
    fn sandboxed(&'vec mut self) -> SandboxMut<'vec, 0, T>;
}

impl<'vec, T> Sandboxed<'vec, T> for Vec<T> where T: 'vec {
    fn sandboxed(&'vec mut self) -> SandboxMut<'vec, 0, T> {
        SandboxMut { vec: self }
    }
}

#[test]
fn push_length_guarantee() {
    let mut vec: Vec<&str> = vec![];
    let sandboxed = vec.sandboxed()
        .push("First");

    println!("{}", sandboxed.first());
    println!("{}", sandboxed.last());
}

#[test]
fn runtime_check_narrowing() {
    let mut vec: Vec<&str> = vec!["one", "two", "three"];
    // the guaranteed length of the sandbox is 1,
    // since we've performed one sandboxed `push` operation
    let sandboxed = vec.sandboxed().push("four");
    match sandboxed.check_len::<3>() {
        // case if the length of the sandbox is at least 3 elements
        // the sandbox instance bound here has a guaranteed length of 3, so we can safely use indices -3 ..= 2
        Ok(at_least_3) => {
            println!("{}", at_least_3.get::<2>()); // safely access the third element (index 2)
            println!("{}", at_least_3.get::<-3>()); // safely access the third-to-last element (index -3 i.e. `len
        }
        // case if the length of the sandbox is less than 3 elements
        // the sandbox instance bound here inherits the original guaranteed length of 1
        Err(default) => {
            println!("vec is less than 3, but at least 1");
            println!("{}", default.get::<0>()); // safely access the first
            println!("{}", default.get::<-1>()); // safely access the last element
        }
    }
}