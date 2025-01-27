use rand::Rng;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use shute::{Buffer, Instance, PowerPreference};
use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::Instant,
};

fn generate_data(dim: usize) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut data: Vec<u32> = (0..dim * dim).map(|_| rng.gen_range(0..100)).collect();
    for i in 0..dim {
        data[dim * i + i] = 0;
    }
    data
}

async fn compute(data: &mut Vec<u32>, dim: u32) {
    let now = Instant::now();
    let instance = Instance::new();
    let device = instance
        .autoselect(PowerPreference::HighPerformance, shute::LimitType::Highest)
        .await
        .unwrap();
    let mut input_buffer = device.create_buffer(
        Some("input"),
        shute::BufferType::StorageBuffer {
            output: true,
            read_only: true,
        },
        shute::BufferInit::WithData(data.clone()),
    );
    let mut output_buffer = device.create_buffer(
        Some("output"),
        shute::BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        shute::BufferInit::<f32>::WithSize(input_buffer.size()),
    );
    let mut dim_buffer = device.create_buffer(
        Some("dim"),
        shute::BufferType::UniformBuffer,
        shute::BufferInit::WithData(dim),
    );
    let groups: Vec<Vec<&mut Buffer>> =
        vec![vec![&mut input_buffer, &mut output_buffer, &mut dim_buffer]];
    let elapsed = now.elapsed();
    println!("[GPU] Buffer setup completed in {:.2?}", elapsed);
    let now = Instant::now();
    let shader = device.create_shader_module(include_str!("shortcut.wgsl"), "main".to_string());
    let elapsed = now.elapsed();
    println!("[GPU] Shader module compiled in {:.2?}", elapsed);
    let now = Instant::now();
    device
        .execute_blocking(&groups, shader, (dim.div_ceil(16), dim.div_ceil(16), 1))
        .await;
    let elapsed = now.elapsed();
    println!("[GPU] Compute completed in: {:.2?}", elapsed);
    let now = Instant::now();
    output_buffer.fetch_data_from_device(&device, data).await;
    let elapsed = now.elapsed();
    println!("[GPU] Data transferred back from GPU in {:.2?}", elapsed);
}
// V1 cpu compute, parallel (sort of)
fn cpu_compute(data: &[u32], dim: u32) -> Vec<u32> {
    let dim = dim as usize;
    let mut transposed = vec![0; data.len()];
    for i in 0..dim {
        for j in 0..dim {
            transposed[dim * j + i] = data[dim * i + j];
        }
    }
    let output: Vec<_> = (0..dim * dim).map(|_| AtomicU32::new(0)).collect();
    (0..dim).into_par_iter().for_each(|i| {
        for j in 0..dim {
            let mut smallest = u32::MAX;
            for k in 0..dim {
                let sum = data[dim * i + k] + transposed[dim * j + k];
                smallest = std::cmp::min(sum, smallest);
            }
            output[dim * i + j].store(smallest, std::sync::atomic::Ordering::Relaxed);
        }
    });
    output
        .par_iter()
        .map(|n| n.load(Ordering::Relaxed))
        .collect()
}

fn main() {
    let test_for_correctness = true;
    let dim = 4000u32;
    let initial_data = generate_data(dim as usize);
    let mut data = initial_data.clone();
    let now = Instant::now();
    pollster::block_on(compute(&mut data, dim));
    let gpu_elapsed = now.elapsed();
    println!("GPU took: {:.2?}", gpu_elapsed);
    if test_for_correctness {
        let now = Instant::now();
        let cpu_result = cpu_compute(&initial_data, dim);
        let cpu_elapsed = now.elapsed();
        println!("CPU took: {:.2?}", cpu_elapsed);
        println!("Verifying correctness...");
        if cpu_result.iter().zip(data.iter()).all(|(a, b)| a == b) {
            println!("Results match.");
        } else {
            println!("Results are inconsistent.");
        }
    }
}
