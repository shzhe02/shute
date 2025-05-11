# Shute

Shute is an abstraction layer/library built on top of [wgpu](https://github.com/gfx-rs/wgpu).

The goal of this library is to simplify the CPU-side code necessary for executing compute shaders.

# Note

This branch of Shute is specifically here to ensure repeatability of results of the benchmarks used in a thesis. No optimizations or API changes are to be made.

However, work to fix a benchmark bug may be performed:

- When running some benchmarks (e.g., the "Powers" benchmark), it may crash every 2 benchmarks due to "memory was not deallocated" (or similar, I can't remember the exact error).
- This error either occurs constantly or never, and I'm not sure why.

# Quickstart

For now, please refer to the examples present in the [examples folder](https://github.com/shzhe02/shute/tree/main/examples).

A more in-depth quickstart guide and improved documentation is coming soon.
