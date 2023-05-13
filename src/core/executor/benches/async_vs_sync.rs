use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, Criterion,
};
use executor::iter::{Iterator, StdIter, Step};

fn fusion_iter<I: for<'iter> executor::iter::Iterator<'iter>>(mut i: I) {
    loop {
        match i.next() {
            Step::NotYet => continue,
            Step::Ready(i) => {
                black_box(i);
            }
            Step::Done(_) => break,
        }
    }
}

fn std_iter<I: std::iter::Iterator>(mut i: I) {
    loop {
        match i.next() {
            Some(i) => {
                black_box(i);
            }
            None => break,
        }
    }
}

#[inline]
async fn async_id<T>(v: T) -> T {
    v
}

fn sync_id<T>(v: T) -> T {
    v
}

fn iterator(c: &mut Criterion) {
    c.bench_function("standard", |b| {
        b.iter(|| {
            std_iter(black_box(
                (0..4096).into_iter().filter(|x| *x % 2 == 0).map(|x| x + 1),
            ))
        })
    });
    c.bench_function("fusion", |b| {
        b.iter(|| {
            fusion_iter(black_box(
                StdIter::from((0..4096).into_iter())
                    .filter(|x| *x % 2 == 0)
                    .map(|x| x + 1),
            ))
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
