use criterion::criterion_main;

mod benchmarks;

criterion_main!(
    benchmarks::io::io,
    benchmarks::shortcut::shortcut,
    benchmarks::shortcut_v0::shortcut,
    benchmarks::shortcut_v1::shortcut,
    benchmarks::shortcut_v2::shortcut,
    benchmarks::powers::powers
);
