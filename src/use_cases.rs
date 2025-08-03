use crate::{GuaranteedLength, NonEmptyOps, Sandboxed};

/// an example use case where we have a sorted list of values, along with a cache of the value bounds.
/// we must make sure that the bounds are updated when we update the values, setting the bounds to `None` when there are no values
struct SortedList {
    sorted_values: Vec<usize>,
    bounds: Option<(usize, usize)>,
}
impl SortedList {
    ///
    fn pop_and_update_bounds(&mut self) -> Option<usize> {
        let popped = self.sorted_values.pop();
        self.bounds = self.sorted_values.first()
            .and_then(|first| {
                self.sorted_values.last()
                    .map(|last| (*first, *last))
            });
        popped
    }

    fn pop_and_update_bounds_sandboxed(&mut self) -> Option<usize> {
        let popped = self.sorted_values.pop();
        self.bounds = self.sorted_values.with_min_length::<1>().map(|sandbox| {
            (*sandbox.first(), *sandbox.last())
        });
        popped
    }

    ///
    fn try_push_and_update_bounds(&mut self, value: usize) -> Result<(), ()> {
        match self.bounds {
            Some((_, max)) if max > value => Err(()),
            _ => {
                self.sorted_values.push(value);
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

    ///
    fn try_push_and_update_bounds_sandboxed(&mut self, value: usize) -> Result<(), ()> {
        match self.bounds {
            Some((_, max)) if max > value => Err(()),
            _ => {
                let sandbox = self.sorted_values.sandboxed().push(value);
                self.bounds = Some((
                    *sandbox.first(),
                    *sandbox.last()
                ));
                Ok(())
            }
        }
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
            .return_get_mut::<-1>()
    }

    /// alternative to `push_and_reference_sandboxed()` using scoped sandbox in a closure
    /// 
    /// similarly, this cannot panic and guarantees a valid result at compile time
    fn push_and_get_mut_sandboxed_scope(&mut self, value: String) -> &mut String {
        self.values.sandboxed_scope(|sandbox| {
            sandbox.push(value).return_get_mut::<-1>()
        })
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
        self.values.swap(0, last_index);
    }

    /// this cannot panic and guarantees a valid result at compile time
    fn swap_insert_front_sandboxed(&mut self, value: String) {
        self.values.sandboxed_scope(|sandbox| {
            sandbox.push(value).swap::<0, -1>()
        })
    }
}

