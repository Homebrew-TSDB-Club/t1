#![feature(generators, generator_trait)]

use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, Criterion,
};

pub struct Map<G, F> {
    stream: G,
    f: F,
}

impl<Arg, G, F, B, T> Generator<Arg> for Map<G, F>
where
    G: Generator<Arg, Yield = Option<T>>,
    F: FnMut(T) -> B,
{
    type Yield = Option<B>;
    type Return = G::Return;

    #[inline]
    fn resume(
        self: std::pin::Pin<&mut Self>,
        arg: Arg,
    ) -> GeneratorState<Self::Yield, Self::Return> {
        let this = unsafe { self.get_unchecked_mut() };
        match unsafe { Pin::new_unchecked(&mut this.stream).resume(arg) } {
            GeneratorState::Yielded(yielded) => match yielded {
                Some(y) => GeneratorState::Yielded(Some((this.f)(y))),
                None => GeneratorState::Yielded(None),
            },
            GeneratorState::Complete(c) => GeneratorState::Complete(c),
        }
    }
}

pub struct Filter<G, F> {
    stream: G,
    f: F,
}

impl<G, F> Generator for Filter<G, F>
where
    G: Generator<()>,
    F: FnMut(&G::Yield) -> bool,
{
    type Yield = Option<G::Yield>;
    type Return = G::Return;

    #[inline]
    fn resume(
        self: std::pin::Pin<&mut Self>,
        _arg: (),
    ) -> GeneratorState<Self::Yield, Self::Return> {
        let this = unsafe { self.get_unchecked_mut() };
        match unsafe { Pin::new_unchecked(&mut this.stream).resume(()) } {
            GeneratorState::Yielded(yielded) => {
                if (this.f)(&yielded) {
                    GeneratorState::Yielded(Some(yielded))
                } else {
                    GeneratorState::Yielded(None)
                }
            }
            GeneratorState::Complete(c) => GeneratorState::Complete(c),
        }
    }
}

#[inline]
fn std_iter<I: std::iter::Iterator>(i: I) {
    for i in i {
        black_box(i);
    }
}

#[inline]
fn generator_iter<I: Generator<Yield = Option<i32>> + Unpin>(mut i: I) {
    while let GeneratorState::Yielded(i) = Pin::new(&mut i).resume(()) {
        black_box(i);
    }
}

#[inline]
async fn async_id<T>(v: T) -> T {
    v
}

#[inline]
fn sync_id<T>(v: T) -> T {
    v
}

fn iterator(c: &mut Criterion) {
    c.bench_function("standard", |b| {
        b.iter(|| std_iter(black_box((0..4096).filter(|x| *x % 2 == 0).map(|x| x + 1))))
    });
    c.bench_function("generator", |b| {
        b.iter(|| {
            generator_iter(black_box(Map {
                stream: Filter {
                    stream: || {
                        let mut i = 0;
                        loop {
                            if i < 4096 {
                                yield i;
                                i += 1;
                            } else {
                                return;
                            }
                        }
                    },
                    f: |x: &i32| *x % 2 == 0,
                },
                f: |x| x + 1,
            }))
        })
    });
}

fn id(c: &mut Criterion) {
    c.bench_function("sync", |b| b.iter(|| sync_id(black_box(1))));
    c.bench_function("async", |b| {
        b.to_async(FuturesExecutor)
            .iter(|| async_id(black_box(black_box(1))))
    });
}

criterion_group!(benches, iterator, id);
criterion_main!(benches);
