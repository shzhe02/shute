//! A very naive implementation of determining the number of iterations a batch of random numbers
//! go through when folling the rules from the Collatz Conjecture.
//!
//! This implementation largely originates from the wgpu example ["hello-compute"](https://github.com/gfx-rs/wgpu/tree/v24/examples/src/hello_compute).

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use shute::{BufferInit, BufferType, Instance, PowerPreference};

fn collatz(data: &Vec<u32>) -> Vec<u32> {
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, shute::LimitType::Highest),
    )
    .expect("Failed to select device");
    let mut input_buffer = device.create_buffer(
        Some("input"),
        BufferType::StorageBuffer {
            output: false,
            read_only: true,
        },
        BufferInit::WithData(data),
    );
    let mut output_buffer = device.create_buffer(
        Some("output"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        BufferInit::<u32>::WithSize(data.len()),
    );
    let shader = device.create_shader_module(include_str!("collatz.wgsl"), "main");
    let groups = vec![vec![&mut input_buffer, &mut output_buffer]];
    device.execute(&groups, shader, [data.len() as u32]);
    let mut output = vec![0; data.len()];
    pollster::block_on(output_buffer.read(&mut output))
        .expect("Failed to fetch data from output buffer");
    output
}

fn generate_data(n: usize) -> Vec<u32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..n).map(|_| rng.r#gen::<u32>()).collect()
}

fn cpu_collatz(data: &Vec<u32>) -> Vec<u32> {
    data.par_iter()
        .map(|num| {
            let mut curr = *num;
            let mut i = 0;
            loop {
                if curr <= 1u32 {
                    break;
                }
                if curr % 2 == 0 {
                    curr >>= 1;
                } else {
                    if curr >= 0x55555555u32 {
                        return 0xffffffffu32;
                    }
                    curr = curr * 3 + 1;
                }
                i += 1;
            }
            i
        })
        .collect()
}

fn main() {
    let data = generate_data(65535);
    let output = collatz(&data);
    let cpu_output = cpu_collatz(&data);
    if output.iter().zip(cpu_output.iter()).all(|(a, b)| a == b) {
        println!("Results match");
    } else {
        println!("Results inconsistent");
    }
}
