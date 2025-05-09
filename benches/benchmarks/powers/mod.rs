use criterion::{BenchmarkId, Criterion, criterion_group};
use shute::{BufferInit, BufferType, Instance, LimitType, PowerPreference, ShaderType};

#[derive(ShaderType)]
struct Input {
    powers: u32,
}

fn compute(data: &mut Vec<f32>, powers: u32) {
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, LimitType::Highest),
    )
    .unwrap();
    let mut input_buffer = device.create_buffer(
        Some("input"),
        BufferType::StorageBuffer {
            output: false,
            read_only: true,
        },
        BufferInit::WithData(&data),
    );
    let mut output_buffer = device.create_buffer(
        Some("output"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        BufferInit::<f32>::WithSize(data.len()),
    );
    let mut param_buffer = device.create_buffer(
        Some("params"),
        BufferType::UniformBuffer,
        BufferInit::WithData(Input { powers }),
    );
    let shader = device
        .create_shader_module_with_workgroup_size(
            include_str!("powers.wgsl"),
            "main",
            [1024],
        )
        .expect("Failed to compile shader module");
    let groups = vec![vec![
        &mut input_buffer,
        &mut output_buffer,
        &mut param_buffer,
    ]];
    device.execute(
        &groups,
        shader,
        [(data.len() as u32).div_ceil(1024)],
    );
    pollster::block_on(output_buffer.read(data)).expect("Could not read output");
}

fn powers_bench(c: &mut Criterion) {
    // let mut group = c.benchmark_group("Powers (with varying size)");
    // for size in (1000000..=10000000).into_iter().step_by(1000000) {
    //     let mut data = (1..=size).map(|num| num as f32).collect();
    //     group.throughput(criterion::Throughput::Elements(size));
    //     group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
    //         b.iter(|| compute(&mut data, 10000));
    //     });
    // }
    // group.finish();
    let mut group = c.benchmark_group("Powers (with varying intensity)");
    for power in (10000..=100000).into_iter().step_by(10000) {
        let mut data = (1..=10000000).map(|num| num as f32).collect();
        group.throughput(criterion::Throughput::Elements(power));
        group.bench_with_input(BenchmarkId::from_parameter(power), &power, |b, _| {
            b.iter(|| compute(&mut data, power as u32));
        });
    }
    group.finish();
}

criterion_group!(powers, powers_bench);
