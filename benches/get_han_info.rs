use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use cindex::han::{Stroke, cjk_info, ordered_strokes};

fn bench_cjk_info(c: &mut Criterion) {
    let h = cjk_info(black_box('汉'));
    assert!(h.is_some());
    let h = h.unwrap();
    assert_eq!(h.mandarin(), Some(("han", 4)));
    assert_eq!(h.glyph_total_strokes(), 5);

    let mut group = c.benchmark_group("cjk info");

    group.bench_function("raw info: U+6C49", |b| b.iter(|| cjk_info(black_box('汉'))));
    group.bench_function("strokes: U+6C49", |b| {
        b.iter(|| {
            let info = cjk_info(black_box('汉')).unwrap();
            black_box(info.glyph_total_strokes())
        })
    });
    group.bench_function("mandarin: U+6C49", |b| {
        b.iter(|| {
            let info = cjk_info(black_box('汉')).unwrap();
            black_box(info.mandarin())
        })
    });

    group.bench_function("raw info: U+33333", |b| {
        b.iter(|| cjk_info(black_box('\u{33333}')))
    });
    group.bench_function("strokes: U+33333", |b| {
        b.iter(|| {
            let info = cjk_info(black_box('\u{33333}')).unwrap();
            black_box(info.glyph_total_strokes())
        })
    });
    group.bench_function("mandarin: U+33333", |b| {
        b.iter(|| {
            let info = cjk_info(black_box('\u{33333}')).unwrap();
            black_box(info.mandarin())
        })
    });

    group.finish();
}

fn bench_ordered_strokes(c: &mut Criterion) {
    use Stroke::*;
    let h = ordered_strokes(black_box('汉'));
    assert!(h.is_some());
    let h = h.unwrap();
    assert_eq!(h.strokes().collect::<Vec<_>>(), vec![S4, S4, S1, S5, S4]);

    c.bench_function("ordered strokes", |b| {
        b.iter(|| ordered_strokes(black_box('汉')))
    });
}

fn bench_strokes_cmp(c: &mut Criterion) {
    c.bench_function("strokes compare", |b| {
        b.iter(|| {
            let h = ordered_strokes(black_box('汉')).unwrap();
            let j = ordered_strokes(black_box('江')).unwrap();
            black_box(h.partial_cmp(&j))
        })
    });
}

criterion_group!(
    get_han_info,
    bench_cjk_info,
    bench_ordered_strokes,
    bench_strokes_cmp
);
criterion_main!(get_han_info);
