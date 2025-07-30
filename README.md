# Preface

While this applies to more than just vectors, I will be focusing on them in my examples and proof of concept, as their API is relatively simple and they are the most ubiquitous and error-prone collection in my experience.

# The problem

I frequently find myself working with vectors that have certain invariant qualities that *I* know, but I cannot tell the compiler without `unsafe`. For example,

```rs
vec.push(value);
vec.first().unwrap()
```

This code will never panic. Nevertheless, we have to either do unnecessary error handling for a None path that will never be returned, or we must use panicking functions like unwrap or expect.

Using panicking functions can lead to a DOS condition if the invariant relationship is broken at some point, and the panicking path becomes accessible.

```rs
fn push_and_ref(vec: &mut Vec<String>, value: String) -> &mut String {
    let value_index = vec.len();
    vec.push(value);
    vec.get_mut(value_index).expect("value should exist at last index")
}
```

Less severely, using error handling for this can be hard to define.

```rs
// we have to return an Option, even though this function will never return `None`
fn push_and_ref(vec: &mut Vec<String>, value: String) -> Option<&mut String> {
    let value_index = vec.len();
    vec.push(value);
    
    // how do we convince the compiler that this is Some(_)?
    // &mut String doesnt have any default value, so we can't without somehow adding a panicking path or `unsafe`
    vec.get_mut(value_index) 
}

// elsewhere

// this may not perform as expected if `add_value()` is reordered
vec.get(value_index).unwrap_or_else(Default::default)
```

Naturally, things like this should be tested, and due diligence should be done.

However, I can point to many, many CVEs to evidence that developers don’t always do their due diligence or test all cases.
Things like this *do* happen, and should be made easier to mitigate.

Having access guarantees checked at compile time would prevent these issues.

# Proposal

The rust type system is robust enough to model most of these guarantees via a Typestate pattern.

This can be done by reserving a mutable reference to a vector, stored in a temporary `Sandbox`.
`Sandbox` implements neither `Copy` nor `Clone`, and so once moved to a closure, it is only accessible within that closure.

This would look something like this
```rs
let mut vec = vec![];
vec.sandboxed_mut(|sandbox| {
    // `sandbox` has unique mutable access to `vec`
})
```

We can then add a constant parameter `MIN_LEN` to sandbox, providing us with some guarantees about the referenced vector.

While this would default to zero, certain operations like `push` could change this value.

These operations would need to take ownership of `sandbox` in order to return a new `sandbox` with a different constant parameter.

```rs
let mut vec = vec![];
vec.sandboxed_mut(|sandbox: Sandbox<0>| {
    let sandbox: Sandbox<1> = sandbox.push(1);
})
```

Finally, we can separate the different vector operations into different traits, implemented only for certain values of `MIN_LEN`.

For example, `pop()`, `first()`, and `last()` all require at least one element, 
so they can be moved to a `NonEmpty` trait that is implemented for `Sandbox<N>` where `N > 0`

Together, this can provide compile-time guarantees for the previous example:
```rs
// this block will not compile if the invariant relationship is broken

vec.sandboxed_mut(|sandbox| {
    sandbox.push(value).get::<
    
});

let value_index = vec.len();
vec.push(value); // will panic if this is reordered
vec.get(value_index).unwrap_or_else(Default::default)
```

The proof of concept implementation is available here:
https://github.com/JamFactoryInc/vec-sandbox





