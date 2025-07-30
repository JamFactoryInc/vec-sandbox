use std::cell::{RefCell, UnsafeCell};
use std::pin::Pin;
use crate::{NonEmptyOps, Sandboxed};

struct Example {
    values: Vec<String>
}

impl Example {
    // may panic if operations are reordered
    fn save_and_reference_panicking(&mut self, value: String) -> &str {
        self.values.push(value);
        self.values.last().expect("value should always exist") 
    }

    // may introduce a silent logic error if operations are reordered
    fn save_and_reference_defaulting(&mut self, value: String) -> &str {
        self.values.push(value);
        self.values.last()
            .map(|str| str.as_str())
            .unwrap_or("err") 
    }

    // defers error handling to the user for value that will never be `None`
    // still may introduce a silent logic error if operations are reordered
    fn save_and_reference_deferring(&mut self, value: String) -> Option<&str> {
        self.values.push(value);
        self.values.last()
            .map(|str| str.as_str()) 
    }

    // this cannot panic and guarantees a valid result at compile time
    // i.e. this will not compile if refactored incorrectly
    fn save_and_reference_sandboxed(&mut self, value: String) -> &str {
        self.values.sandboxed()
            .push(value)
            .return_get::<-1>()
    }
}
