use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use luckyapi::handlers::zip_handler::{
    async_walk_dir, walk_dir_sync, walkdir_sync_v2,
};

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use std::path::{Path, PathBuf};

fn async_walk_dir_benchmark(c: &mut Criterion) {
    let dir: PathBuf =
        black_box(Path::new("/home/csh0101/lab/lucky-x-server").into());

    c.bench_with_input(
        BenchmarkId::new("async walkdir", "dir"),
        &dir,
        |b, dir| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async_walk_dir(dir.clone()))
        },
    );

    // c.bench_function("walk dir", |b| j);
}

fn sync_walk_dir_benchmark(c: &mut Criterion) {
    let dir: PathBuf =
        black_box(Path::new("/home/csh0101/lab/lucky-x-server").into());
    c.bench_function("sycn walkdir", |b| {
        b.iter(|| walk_dir_sync(black_box(dir.clone())))
    });
}

fn sync_walk_dir_benchmark_v2(c: &mut Criterion) {
    let dir: PathBuf =
        black_box(Path::new("/home/csh0101/lab/lucky-x-server").into());
    c.bench_function("sycn walkdir v2 by walkdir crate", |b| {
        b.iter(|| walkdir_sync_v2(black_box(dir.clone())))
    });
}

criterion_group!(
    benches,
    sync_walk_dir_benchmark,
    sync_walk_dir_benchmark_v2,
    async_walk_dir_benchmark,
);
criterion_main!(benches);
