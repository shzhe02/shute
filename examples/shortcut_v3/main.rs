use rand::Rng;
use shute::{Buffer, BufferInit, BufferType, Instance, LimitType, PowerPreference, ShaderType};

fn generate_data(dim: usize) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut data: Vec<u32> = (0..dim * dim).map(|_| rng.gen_range(0..100)).collect();
    for i in 0..dim {
        data[dim * i + i] = 0;
    }
    data
}

#[derive(ShaderType)]
struct Input {
    dim: u32,
    nn: u32,
}

fn compute(data: &mut Vec<u32>, dim: u32) {
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
        BufferInit::<u32>::WithSize((nn * nn) as usize),
    );
    let mut input_buffer_t = device.create_buffer(
        Some("input_t"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        BufferInit::<u32>::WithSize((nn * nn) as usize),
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
    device.execute_blocking(&groups, padding_shader, (1, nn, 1));
    let shader = device.create_shader_module(include_str!("shortcut.wgsl"), "main");
    device.execute_blocking(&groups, shader, (nn / 64, nn / 64, 1));
    pollster::block_on(output_buffer.fetch_data_from_device(data));
}

// V1 cpu compute, parallel (sort of)
fn cpu_compute(data: &[u32], dim: u32) -> Vec<u32> {
    use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
    use std::sync::atomic::{AtomicU32, Ordering};
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
    use std::time::Instant;
    let test_for_correctness = true;
    let dim = 1000u32;
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
