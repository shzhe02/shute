use std::time::Duration;

use criterion::{Criterion, criterion_group};
use rand::{Rng, thread_rng};
use shute::{Buffer, BufferInit, BufferType, Device, Instance, LimitType};
use wgpu::PowerPreference;

fn generate_data(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    (0..size).map(|_| rng.r#gen()).collect()
}

fn initialize_gpu_buffer<'a>(device: &'a Device, data: &'a Vec<u32>) -> Buffer<'a> {
    let buffer = device.create_buffer(
        Some("test_send"),
        BufferType::StorageBuffer {
            output: true,
            read_only: true,
        },
        BufferInit::WithData(data),
    );
    device.synchronize();
    buffer
}

fn pull_data_from_buffer(buffer: &mut Buffer, output: &mut Vec<u32>) {
    pollster::block_on(buffer.read(output)).expect("unable to read buffer");
}

fn io_oneshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("CPU-GPU Data Transfer (One-Shot)");
    group
        .sample_size(10)
        .noise_threshold(0.15)
        .measurement_time(Duration::from_secs(10));
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, LimitType::Downlevel),
    )
    .unwrap();
    // Benchmarks for sending data from CPU to GPU

    let size = device.limits().max_buffer_size as usize / size_of::<u32>();
    device.override_staging_size((size * size_of::<u32>()) as u32);
    dbg!(size);
    let mut buffer: Option<Buffer> = None;
    let data = generate_data(size);

    group.bench_function("Buffer Initialization", |b| {
        b.iter(|| {
            buffer = Some(initialize_gpu_buffer(&device, &data));
        })
    });

    // Benchmarks for pulling data from GPU to CPU
    let mut output = vec![0; size];
    group.bench_function("Reading Data from GPU Buffer", |b| {
        b.iter(|| pull_data_from_buffer(buffer.as_mut().unwrap(), &mut output))
    });
}
fn io_tenshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("CPU-GPU Data Transfer (10-Shot)");
    group
        .sample_size(10)
        .noise_threshold(0.15)
        .measurement_time(Duration::from_secs(10));
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, LimitType::Downlevel),
    )
    .unwrap();

    // Benchmarks for sending data from CPU to GPU

    let n = 10;

    let size = 1000;
    let mut buffers = vec![];
    let data = generate_data(size);

    group.bench_function("Buffer Initialization", |b| {
        b.iter(|| {
            (0..n).for_each(|_| {
                buffers.push(initialize_gpu_buffer(&device, &data));
            });
        });
    });

    // Benchmarks for pulling data from GPU to CPU
    let mut output = vec![0; size];
    group.bench_function("Reading Data from GPU Buffer", |b| {
        b.iter(|| {
            buffers
                .iter_mut()
                .for_each(|buffer| pull_data_from_buffer(buffer, &mut output))
        });
    });
}

criterion_group!(io, io_oneshot, io_tenshot);
