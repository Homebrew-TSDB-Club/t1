#![feature(portable_simd)]

use std::num::Wrapping;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn double(list: &[i32]) {
    let (prefix, middle, suffix) = list.as_simd::<8>();

    prefix
        .iter()
        .copied()
        .zip(prefix.iter().copied())
        .for_each(|(origin, shift)| {
            black_box((Wrapping(origin) + Wrapping(shift)).0);
        });
    middle
        .iter()
        .copied()
        .zip(middle.iter().copied())
        .for_each(|(origin, shift)| {
            black_box(origin + shift);
        });
    suffix
        .iter()
        .copied()
        .zip(suffix.iter().copied())
        .for_each(|(origin, shift)| {
            black_box((Wrapping(origin) + Wrapping(shift)).0);
        });
}

fn no_simd_double(list: &[i32]) {
    list.iter()
        .copied()
        .zip(list.iter().copied())
        .for_each(|(list, shift)| {
            black_box((Wrapping(list) + Wrapping(shift)).0);
        });
}

fn simd(c: &mut Criterion) {
    let mut vec = (0..255).collect::<Vec<_>>();
    vec.extend((0..255).collect::<Vec<_>>());
    c.bench_function("no simd", |b| b.iter(|| no_simd_double(black_box(&vec))));
    c.bench_function("simd", |b| b.iter(|| double(black_box(&vec))));
}

criterion_group!(benches, simd);
criterion_main!(benches);
