use rand::Rng;
use shute::{Buffer, Instance, PowerPreference};
use std::time::Instant;

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

async fn compute(data: &Vec<u32>, dim: u32) -> Vec<u32> {
    let now = Instant::now();
    let instance = Instance::new();
    let device = instance
        .autoselect(PowerPreference::HighPerformance, shute::LimitType::Highest)
        .await
        .unwrap();
    dbg!(device.limits());
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
        shute::BufferInit::WithData(dim),
    );
    let mut groups: Vec<Vec<&mut Buffer>> =
        vec![vec![&mut input_buffer, &mut output_buffer, &mut dim_buffer]];
    let elapsed = now.elapsed();
    println!("[GPU] Buffer setup completed in {:.2?}", elapsed);
    let now = Instant::now();
    device.send_all_data_to_device(&groups);
    let elapsed = now.elapsed();
    println!("[GPU] Data transferred to GPU in {:.2?}", elapsed);
    let now = Instant::now();
    let shader = device.create_shader_module(include_str!("shortcut.wgsl"), "main".to_string());
    let elapsed = now.elapsed();
    println!("[GPU] Shader module compiled in {:.2?}", elapsed);
    let now = Instant::now();
    device
        .execute_blocking(&groups, shader, (divup(dim, 16), divup(dim, 16), 1))
        .await;
    let elapsed = now.elapsed();
    println!("[GPU] Compute completed in: {:.2?}", elapsed);
    let now = Instant::now();
    device.fetch_all_data_from_device(&mut groups).await;
    let elapsed = now.elapsed();
    println!("[GPU] Data transferred back from GPU in {:.2?}", elapsed);
    let now = Instant::now();
    let output: Vec<u32> =
        bytemuck::cast_slice(&output_buffer.read_output_data().as_ref().unwrap()).to_vec();
    let elapsed = now.elapsed();
    println!("[GPU] Casted data back into Vec<u32> in {:.2?}", elapsed);
    output
}
// V1 cpu compute
fn cpu_compute(data: &Vec<u32>, dim: u32) -> Vec<u32> {
    let dim = dim as usize;
    let mut transposed = vec![0; data.len()];
    for i in 0..dim {
        for j in 0..dim {
            transposed[dim * j + i] = data[dim * i + j];
        }
    }
    let mut output = vec![0; dim * dim];
    for i in 0..dim {
        for j in 0..dim {
            let mut smallest = u32::MAX;
            for k in 0..dim {
                let sum = data[dim * i + k] + transposed[dim * j + k];
                smallest = std::cmp::min(sum, smallest);
            }
            output[dim * i + j] = smallest;
        }
    }
    output
}

fn main() {
    let test_for_correctness = false;
    let dim = 10000u32;
    let data = generate_data(dim as usize);
    let now = Instant::now();
    let gpu_result = pollster::block_on(compute(&data, dim));
    let gpu_elapsed = now.elapsed();
    println!("GPU took: {:.2?}", gpu_elapsed);
    if test_for_correctness {
        let now = Instant::now();
        let cpu_result = cpu_compute(&data, dim);
        let cpu_elapsed = now.elapsed();
        println!("CPU took: {:.2?}", cpu_elapsed);
        println!("Verifying correctness...");
        if cpu_result
            .iter()
            .zip(gpu_result.iter())
            .all(|(a, b)| a == b)
        {
            println!("Results match.");
        } else {
            println!("Results are inconsistent.");
        }
    }
}
