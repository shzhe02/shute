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

fn pull_data_from_buffer(device: &Device, buffer: &mut Buffer) {
    pollster::block_on(buffer.fetch_data_from_device(device));
    let _output: Vec<u32> =
        bytemuck::cast_slice(buffer.read_output_data().as_ref().unwrap()).to_vec();
}

fn benchmark_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("CPU-GPU IO");
    group.sample_size(10).noise_threshold(0.15);
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, LimitType::Highest),
    )
    .unwrap();

    // Benchmarks for sending data from CPU to GPU

    for i in (1..10).map(|e| e * 10000000) {
        group.bench_with_input(BenchmarkId::new("Sending", i), &generate_data(i), |b, i| {
            b.iter(|| initialize_gpu_buffer(&device, i))
        });
    }

    // Benchmarks for pulling data from GPU to CPU
    for i in (1..10).map(|e| e * 10000000) {
        let mut buffer = device.create_buffer(
            Some("test_pull"),
            BufferType::StorageBuffer {
                output: true,
                read_only: true,
            },
            BufferInit::WithSize::<Vec<u32>>(i),
        );
        group.bench_function(BenchmarkId::new("Pulling", i), |b| {
            b.iter(|| pull_data_from_buffer(&device, &mut buffer))
        });
    }
}

criterion_group!(io, benchmark_io);
