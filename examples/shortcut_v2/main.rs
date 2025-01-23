use encase::ShaderType;
use rand::Rng;
use shute::{Buffer, Instance, PowerPreference};

fn generate_data(dim: usize) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut data: Vec<u32> = (0..dim * dim).map(|_| rng.gen_range(0..100)).collect();
    for i in 0..dim as usize {
        data[dim * i + i] = 0;
    }
    data
}

fn divup(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

fn roundup(a: u32, b: u32) -> u32 {
    divup(a, b) * b
}

#[derive(ShaderType)]
struct Input {
    dim: u32,
    nn: u32,
}

async fn compute(data: &Vec<u32>, dim: u32, nn: u32) -> Vec<u32> {
    let instance = Instance::new();
    let device = instance
        .autoselect(PowerPreference::HighPerformance)
        .await
        .unwrap();

    let mut input_buffer = device.create_buffer(
        Some("input"),
        shute::BufferType::StorageBuffer {
            output: true,
            read_only: true,
        },
        shute::BufferInit::WithData(data),
    );
    let mut output_buffer = device.create_buffer(
        Some("output"),
        shute::BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        shute::BufferInit::<Vec<f32>>::WithSize(input_buffer.size()),
    );
    let mut dim_buffer = device.create_buffer(
        Some("dim"),
        shute::BufferType::UniformBuffer,
        shute::BufferInit::WithData(Input { dim, nn }),
    );
    let mut groups: Vec<Vec<&mut Buffer>> =
        vec![vec![&mut input_buffer, &mut output_buffer, &mut dim_buffer]];
    device.send_all_data_to_device(&groups);
    let shader = device.create_shader_module(include_str!("shortcut.wgsl"), "main".to_string());
    device
        .execute(&groups, shader, (divup(dim, 16), divup(dim, 16), 1))
        .await;
    device.fetch_all_data_from_device(&mut groups).await;
    let output: Vec<u32> =
        bytemuck::cast_slice(&output_buffer.read_output_data().as_ref().unwrap()).to_vec();

    output
}

fn main() {
    use std::time::Instant;
    let dim = 4000u32;
    let nn = roundup(dim, 64);
    let data = generate_data(dim as usize);
    let now = Instant::now();
    pollster::block_on(compute(&data, dim, nn));
    let gpu_elapsed = now.elapsed();
    println!("GPU took: {:.2?}", gpu_elapsed);
}
