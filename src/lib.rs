#![feature(generic_const_exprs)]

struct SafeLength<const MIN_LEN: usize>(usize);

struct Sandbox<'a, const MIN_LEN: usize, T> {
    vec: &'a mut Vec<T>
}

impl<'a, const MIN_LEN: usize, T> Sandbox<'a, MIN_LEN, T> {
    fn len(&self) -> SafeLength<MIN_LEN> {
        SafeLength(self.vec.len())
    }

    fn push(mut self, value: T) -> Sandbox<'a, { MIN_LEN + 1 }, T> {
        self.vec.push(value);
        Sandbox {
            vec: self.vec
        }
    }
}

struct GenericNumber<const N: usize>;

trait NonZero<const TRUE: bool> {}
impl<const N: usize> NonZero<{ N > 0}> for GenericNumber<N> {}

trait NonEmptyOps<'a, const MIN_LEN: usize, T> {
    fn first(&self) -> &T;
    fn last(&self) -> &T;

    fn pop(self) -> (Sandbox<'a, { MIN_LEN - 1 }, T>, T);
}

impl<'a, const MIN_LEN: usize, T> NonEmptyOps<'a, MIN_LEN, T> for Sandbox<'a, MIN_LEN, T> where GenericNumber<MIN_LEN>: NonZero<true> {
    fn first(&self) -> &T {
        self.vec.first().unwrap()
    }

    fn last(&self) -> &T {
        self.vec.last().unwrap()
    }

    fn pop(self) -> (Sandbox<'a, {MIN_LEN - 1}, T>, T) {
        let popped = self.vec.pop().unwrap();
        (
            Sandbox { vec: self.vec },
            popped
        )
    }
}

trait SandboxedVec<T> {
    fn sandboxed<'a, F: Fn(Sandbox<'a, 0, T>)>(&'a mut self, safe_accessor: F) where T: 'a;
}
impl<T> SandboxedVec<T> for Vec<T> {
    fn sandboxed<'a, F: Fn(Sandbox<'a, 0, T>)>(&'a mut self, safe_accessor: F) {
        safe_accessor(Sandbox { vec: self })
    }
}

#[test]
fn it_works() {
    let mut vec = Vec::default();
    vec.sandboxed(|sandbox| {
        let sandbox = sandbox.push(1);

        println!("{}", sandbox.first());
    });
}