use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, Criterion,
};
use executor::stream::{IterStream, Iterator, Step};

async fn test_async<I: executor::stream::Iterator>(mut i: I) {
    loop {
        match i.next().await {
            Step::NotYet => continue,
            Step::Ready(i) => {
                let _ = i;
            }
            Step::Done => break,
        }
    }
}

fn test_sync<I: std::iter::Iterator>(mut i: I) {
    loop {
        match i.next() {
            Some(i) => {
                let _ = i;
            }
            None => break,
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sync", |b| {
        b.iter(|| {
            test_sync(black_box(
                (0..4096).into_iter().filter(|x| *x % 2 == 0).map(|x| x + 1),
            ))
        })
    });
    c.bench_function("async", |b| {
        b.to_async(FuturesExecutor).iter(|| {
            test_async(black_box(
                IterStream::from((0..4096).into_iter())
                    .filter(|x| *x % 2 == 0)
                    .map(|x| x + 1),
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
