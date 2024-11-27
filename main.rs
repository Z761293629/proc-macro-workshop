use derive_debug::CustomDebug;
use std::fmt::Debug;
use std::marker::PhantomData;

type S = String;

#[derive(CustomDebug)]
struct GeekKindergarten<T, U, V, W> {
    blog: T,
    ideawand: PhantomData<U>,
    com: U,
    foo: PhantomData<V>,
    bar: Vec<W>,
    #[debug = "0b{:08b}"]
    bitmask: u8,
    name: S,
}

fn assert_debug<F: Debug>() {}

fn main() {
    // Does not implement Debug.
    struct NotDebug;

    assert_debug::<PhantomData<NotDebug>>();
    assert_debug::<GeekKindergarten<u32, u16, NotDebug, u16>>();
}
