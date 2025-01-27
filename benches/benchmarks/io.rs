use std::time::Duration;

use criterion::{criterion_group, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};
use shute::{Buffer, BufferInit, BufferType, Device, Instance, LimitType};
use wgpu::PowerPreference;

fn generate_data(size: usize) -> Vec<u32> {
    let mut rng = thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

fn initialize_gpu_buffer(device: &Device, data: &Vec<u32>) {
    let buffer = device.create_buffer(
        Some("test_send"),
        BufferType::StorageBuffer {
            output: false,
            read_only: true,
        },
        BufferInit::WithData(data),
    );
    buffer.send_data_to_device(device);
    device.block_until_complete();
}

fn pull_data_from_buffer(device: &Device, buffer: &mut Buffer, output: &mut Vec<u32>) {
    pollster::block_on(buffer.fetch_data_from_device(device, output));
}

fn benchmark_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("CPU-GPU IO");
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

    group.bench_with_input(
        BenchmarkId::new("Buffer Initialization", size),
        &generate_data(size),
        |b, i| b.iter(|| initialize_gpu_buffer(&device, i)),
    );

    // Benchmarks for pulling data from GPU to CPU
    let mut buffer = device.create_buffer(
        Some("test_pull"),
        BufferType::StorageBuffer {
            output: true,
            read_only: true,
        },
        BufferInit::WithSize::<u32>(size as u32),
    );
    let mut output = vec![0; size];
    group.bench_function("Reading Data from GPU Buffer", |b| {
        b.iter(|| pull_data_from_buffer(&device, &mut buffer, &mut output))
    });
}

criterion_group!(io, benchmark_io);
