use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use cindex::han::{cantonese_index, mandarin_index};

fn cantonese_search(c: &mut Criterion) {
    c.bench_function("cantonese search", |b| {
        b.iter(|| cantonese_index(black_box("nong")))
    });
}

fn mandarin_search(c: &mut Criterion) {
    c.bench_function("mandarin search", |b| {
        b.iter(|| mandarin_index(black_box("nong")))
    });
}

criterion_group!(readings_map, cantonese_search, mandarin_search);
criterion_main!(readings_map);
