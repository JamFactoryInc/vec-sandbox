# Preface

> Note:
>
> The proof of concept implementation is available here: https://github.com/JamFactoryInc/vec-sandbox
>
> See `src/use_cases.rs` for concrete examples and comparisons

While this applies to more than just vectors, I will be focusing on them in my examples and proof of concept, as their API is relatively simple and they are the most ubiquitous and error-prone collection in my experience.

# The problem


I frequently find myself working with vectors that have certain invariant qualities that *I* know, but I cannot tell the compiler without `unsafe`. 

For example,

```rust
fn do_something(vec: Vec<_>) {
    vec.push(value);
    vec.first().expect("value should exist at first index")
}
```

This code will never panic as-is. Nevertheless, we have to either do unnecessary error handling for a None path that will never be returned, or we must use panicking functions like `unwrap` or `expect`.

Using panicking functions can lead to a DOS condition if the invariant relationship is broken at some point, making the panicking path accessible.

```rust
fn push_and_ref(vec: &mut Vec<String>, value: String) -> &mut String {
    // this is a minimal case, but invariant relationships like this often exist in larger contexts that are more difficult to reason about

    // if removed / reordered, the line below may panic, depending on the state of `vec`
    vec.push(value); 
    vec.last_mut().expect("value should exist at last index")
}
```

We can try to add error handling, but in some cases, this can be more trouble than it's worth.

How do you handle a value that should never exist?

How do you test a code path that never executes?

```rust
// we have to return an Option, even though this function will never return `None`
fn push_and_ref(vec: &mut Vec<String>, value: String) -> Option<&mut String> {
    vec.push(value);
    
    // how do we convince the compiler that `last_mut()` is always present?
    // additionally, a `&mut String` can only be returned here if it outlives our elided `&'1 mut Vec<String>` lifetime.
    // since we cannot access any reference to a vector without either panicking or somehow unwrapping an `Option`/`Result`, 
    // the only valid return value would be a `&'static mut String`, but mutable statics are unsafe.

    // all this to say, I can't think of a way to return a valid `&mut String` from this function without a panicking path or using `unsafe`, 
    // so we must defer the error handling to the caller, but now we've only multiplied & distributed our problems.
    vec.last_mut(value_index) 
}

// elsewhere

vec.push_and_ref(some_value).unwrap_or_else(|| {
    // what do we do here?
    // This code will *probably* never be executed, but this isn't guaranteed
})

```

What I usually see in these situations is `expect` or `unwrap`.

This is fine if properly tested and due diligence is performed. 
However, I can point to many, many CVEs to evidence that developers don’t always do their due diligence or test all the cases.

Having element access guarantees checked at compile time would prevent these issues;
we need to defer enforcement of these invariant relationships to the compiler.

# Proposal

The rust type system is robust enough to model most of these guarantees via a Typestate pattern.

This can be done by reserving a mutable reference to a vector, stored in an owned `SandboxMut` instance.


This would look something like this
```rust
let mut vec = vec![];
vec.sandboxed_scope(|sandbox| {
    // `sandbox` has unique mutable access to `vec`
})
```

We can then add a constant parameter `MIN_LEN` to sandbox, providing us with some guarantees about the referenced vector.

While this would default to zero, certain operations like `push` could change this value.

These operations would need to take ownership of `sandbox` in order to return a new `sandbox` with a different constant parameter.

```rust
let mut vec = vec![];
vec.sandboxed_scope(|sandbox: SandboxMut<0, _>| {
    let sandbox: SandboxMut<1, _> = sandbox.push(1); // the new sandbox has a minimum length of 1
})
```

Finally, we can separate the different vector operations into different traits, implemented only for certain values of `MIN_LEN`.

For example, `pop()`, `first()`, and `last()` all require at least one element, 
so they can be moved to a `NonEmpty` trait that is implemented for `SandboxMut<N, T>` where `N > 0`

Together, this can provide compile-time guarantees for the previous example:
```rust
// we can safely return a `&mut String` using sandboxing
// this function will not compile if the invariant relationship is broken
fn push_and_ref_sandboxed(vec: &mut Vec<String>, value: String) -> &mut String {
    // we can also just obtain the sandbox directly, without wrapping it in a closure
    vec.sandboxed() 
        .push(value)
        // `release_get` is like `get`, but takes ownership of the sandbox, allowing its returned reference to outlive the sandbox itself
        // -1 is a negative index, indicating the last element of the vector
        .release_get_mut::<-1>() 
}
```





