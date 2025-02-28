//! A near-optimal implementation of the min-plus matrix multiplication algorithm
//! (which I call "shortcut algorithm" for convenience).
//!
//! Implementation is more or less from [Programming Parallel Computers, Chapter 4, V3](https://ppc.cs.aalto.fi/ch4/v3/).
//! CPU reference function is derived from [Chapter 2, V2](https://ppc.cs.aalto.fi/ch2/v2/).

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
    pollster::block_on(output_buffer.read(data));
}

// V1 cpu compute, parallel (sort of)
fn cpu_compute(data: &[f32], dim: u32) -> Vec<f32> {
    use atomic_float::AtomicF32;
    use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
    use std::sync::atomic::Ordering;

    let dim = dim as usize;
    let mut transposed = vec![0.0; data.len()];
    for i in 0..dim {
        for j in 0..dim {
            transposed[dim * j + i] = data[dim * i + j];
        }
    }
    let output: Vec<_> = (0..dim * dim).map(|_| AtomicF32::new(0.0)).collect();
    (0..dim).into_par_iter().for_each(|i| {
        for j in 0..dim {
            let mut smallest = f32::MAX;
            for k in 0..dim {
                let sum = data[dim * i + k] + transposed[dim * j + k];
                smallest = if smallest < sum { smallest } else { sum };
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
    use std::time::Instant;
    let test_for_correctness = false;
    let dim = 6300u32;
    let initial_data = generate_data(dim as usize);
    let mut data = initial_data.clone();
    let now = Instant::now();
    compute(&mut data, dim);
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
            println!("Initial Data:");
            for line in initial_data.chunks(dim as usize) {
                println!("{:?}", line);
            }
            println!("========================================");
            println!("CPU result:");
            for line in cpu_result.chunks(dim as usize) {
                println!("{:?}", line);
            }
            println!("GPU result:");
            for line in data.chunks(dim as usize) {
                println!("{:?}", line);
            }
        }
    }
}
