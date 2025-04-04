use criterion::criterion_main;

mod benchmarks;

criterion_main!(
    benchmarks::io::io,
    benchmarks::shortcut::shortcut,
    benchmarks::powers::powers
);
