use crate::{GuaranteedLength, NonEmptyOps, Sandboxed};

/// an example use case where we have a sorted list of values, along with a cache of the value bounds.
/// we must make sure that the bounds are updated when we update the values, setting the bounds to `None` when there are no values
struct SortedList {
    sorted_values: Vec<usize>,
    bounds: Option<(usize, usize)>,
}
impl SortedList {
    /// removes the last (largest) element and updates the cached bounds of this list, returning the popped value
    fn pop_and_update_bounds(&mut self) -> Option<usize> {
        self.sorted_values.pop().inspect(|_| {
            // we know that if `first` exists, then `last` must also exist, but the compiler doesn't have a way to express this guarantee
            // instead have to do this `Option` gymnastics to turn `(Option<usize>, Option<usize>)` into `Option<(usize, usize)>`
            self.bounds = self.sorted_values.first()
                .and_then(|first| {
                    self.sorted_values.last()
                        .map(|last| (*first, *last))
                });
        })
    }

    /// removes the last (largest) element and updates the cached bounds of this list, returning the popped value
    /// 
    /// this sandboxed implementation is made simpler by leveraging the necessary coexistence of the `first()` and `last()` values in our list
    fn pop_and_update_bounds_sandboxed(&mut self) -> Option<usize> {
        self.sorted_values.pop().inspect(|_| {
            // here, we have a compile-time guarantee that `first()` and `last()` necessarily coexist, and are thus directly convertible to `Option<(usize, usize)>`
            self.bounds = self.sorted_values.as_non_empty().map(|sandbox| {
                (*sandbox.first(), *sandbox.last())
            });
        })
    }

    /// tries to add a value to the end of the sorted list, updating the cached bounds if the operation is performed
    /// 
    /// returns an `Err` if the given value cannot be appended to the list while maintaining a natural ordering
    fn try_push_and_update_bounds(&mut self, value: usize) -> Result<(), ()> {
        match self.bounds {
            Some((_, max)) if max > value => Err(()),
            _ => {
                self.sorted_values.push(value);
                // we know that `sorted_values` must be non-empty due to the preceding `push()`, and can safely use `expect()` here on `first()` and `last()
                // however, this introduces a panicking path into our code that may become accessible if refactored improperly
                // this approach requires an additional invariant to be tracked by the developer(s), and leaves it up to them to ensure the validity of the program at runtime
                // 
                // it's also just ugly boilerplate
                let new_min = self.sorted_values.first().expect("first value should exist after push");
                let new_max = self.sorted_values.last().expect("last value should exist after push");
                self.bounds = Some((
                    *new_min,
                    *new_max
                ));
                Ok(())
            }
        }
    }

    /// tries to add a value to the end of the sorted list, updating the cached bounds if the operation is performed
    ///
    /// returns an `Err` if the given value cannot be appended to the list while maintaining a natural ordering
    /// 
    /// this sandboxed implementation is made both simpler and safer by leveraging compile-time length guarantees
    fn try_push_and_update_bounds_sandboxed(&mut self, value: usize) -> Result<(), ()> {
        match self.bounds {
            Some((_, max)) if max > value => Err(()),
            _ => {
                let sandbox = self.sorted_values.sandboxed().push(value);
                
                // here, the compiler can guarantee that `sandbox` is non-empty, and we can allow direct, unchecked access to `first()` and `last()`
                // furthermore, this prevents improper refactoring, as if `push` is removed or re-ordered, the following code will not compile, 
                // as it no longer satisfies the non-zero `MIN_LEN` requirement for `first()` and `last()` to be implemented.

                // try commenting out the following line to watch this in action:
                // let (sandbox, _) = sandbox.pop();
                
                self.bounds = Some((
                    *sandbox.first(),
                    *sandbox.last()
                ));
                Ok(())
            }
        }
    }
}

/// an example use case where we need to quickly insert a value at the beginning of the vector, moving the value that was at the beginning to the end
struct SwapInsertFront {
    values: Vec<String>
}
impl SwapInsertFront {
    /// may panic if operations are reordered
    fn swap_insert_front_panicking(&mut self, value: String) {
        let last_index = self.values.len();
        self.values.push(value);
        // `swap` panics if the indices are not valid, and offers no safe non-panicking alternative
        self.values.swap(0, last_index);
    }

    /// this cannot panic and guarantees a valid result at compile time
    fn swap_insert_front_sandboxed(&mut self, value: String) {
        self.values.sandboxed_scope(|sandbox| {
            // sandbox's `swap` cannot panic, as indices are checked at compile-time
            sandbox.push(value).swap::<0, -1>()
        })
    }
}

/// an example use case where we need to push value to a contained vector and return a mutable reference to the pushed value
struct PushAndGetMut {
    values: Vec<String>
}

impl PushAndGetMut {
    /// may panic if operations are reordered
    fn push_and_get_mut_panicking(&mut self, value: String) -> &mut String {
        self.values.push(value);
        self.values.last_mut().expect("value should always exist") 
    }

    /// defers error handling to the user for value that will never be `None`, resulting in dead / untestable code
    fn push_and_get_mut_deferring(&mut self, value: String) -> Option<&mut String> {
        self.values.push(value);
        self.values.last_mut()
    }

    /// this cannot panic and guarantees a valid result at compile time
    fn push_and_get_mut_sandboxed(&mut self, value: String) -> &mut String {
        self.values.sandboxed()
            .push(value)
            .release_get_mut::<-1>()
    }

    /// alternative to `push_and_reference_sandboxed()` using scoped sandbox in a closure
    /// 
    /// similarly, this cannot panic and guarantees a valid result at compile time
    fn push_and_get_mut_sandboxed_scope(&mut self, value: String) -> &mut String {
        self.values.sandboxed_scope(|sandbox| {
            sandbox.push(value).release_get_mut::<-1>()
        })
    }
}

