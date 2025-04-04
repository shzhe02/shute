use criterion::{BenchmarkId, Criterion, criterion_group};
use rand::Rng;
use shute::{Buffer, BufferInit, BufferType, Instance, LimitType, PowerPreference, ShaderType};

fn generate_data(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let mut data: Vec<f32> = (0..dim * dim).map(|_| rng.r#gen::<f32>()).collect();
    for i in 0..dim {
        data[dim * i + i] = 0.0;
    }
    data
}

#[derive(ShaderType)]
struct Input {
    dim: u32,
    nn: u32,
}

fn compute(data: &mut Vec<f32>, dim: u32) {
    let nn = dim.div_ceil(64) * 64;
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, LimitType::Highest),
    )
    .unwrap();

    let mut input_buffer = device.create_buffer(
        Some("input"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        BufferInit::<f32>::WithSize((nn * nn) as usize),
    );
    let mut input_buffer_t = device.create_buffer(
        Some("input_t"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        BufferInit::<f32>::WithSize((nn * nn) as usize),
    );
    let mut output_buffer = device.create_buffer(
        Some("output"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        BufferInit::WithData(data.to_owned()),
    );
    let mut param_buffer = device.create_buffer(
        Some("params"),
        BufferType::UniformBuffer,
        BufferInit::WithData(Input { dim, nn }),
    );
    let groups: Vec<Vec<&mut Buffer>> = vec![vec![
        &mut input_buffer,
        &mut input_buffer_t,
        &mut output_buffer,
        &mut param_buffer,
    ]];
    let padding_shader = device.create_shader_module(include_str!("padding.wgsl"), "main");
    device.execute(&groups, padding_shader, [1, nn]);
    let shader = device.create_shader_module(include_str!("shortcut.wgsl"), "main");
    device.execute(&groups, shader, [nn / 64, nn / 64]);
    pollster::block_on(output_buffer.read(data)).expect("Failed to fetch data from output buffer");
}

fn shortcut_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Shortcut");
    for dim in [1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000].iter() {
        let mut data = generate_data(*dim as usize);
        group.throughput(criterion::Throughput::Elements(dim * dim));
        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |b, &dim| {
            b.iter(|| compute(&mut data, dim.try_into().unwrap()));
        });
    }
    group.finish();
}

criterion_group!(shortcut, shortcut_bench);
