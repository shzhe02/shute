//! A naive implementation of reduction, roughly matching the implementation of Reduction #1
//! from the CUDA presentation (?) [Optimizing Parallel Reduction in CUDA](https://developer.download.nvidia.com/assets/cuda/files/reduction.pdf)

use shute::{BufferType, Instance, LimitType, PowerPreference};

fn generate_random_data(n: usize) -> Vec<i32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..n).map(|_| rng.gen_range(-10..10)).collect()
}

fn compute(data: &Vec<i32>) -> i32 {
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, LimitType::Highest),
    )
    .unwrap();

    // We are setting both the "input" and "output" buffers to be mutable output storage buffers
    // because we are going to continuously swap them every dispatch.
    let mut buffer_a = device.create_buffer(
        Some("input"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        shute::BufferInit::WithData(data),
    );
    let mut buffer_b = device.create_buffer(
        Some("output"),
        BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        shute::BufferInit::<i32>::WithSize(data.len().div_ceil(128)),
    );
    let mut buffer_n = device.create_buffer(
        Some("n"),
        BufferType::UniformBuffer,
        shute::BufferInit::WithData(data.len() as u32),
    );
    let mut remaining = data.len();
    let mut count = 0;
    while remaining > 1 {
        let groups = vec![vec![&mut buffer_a, &mut buffer_b, &mut buffer_n]];
        let shader = device.create_shader_module(include_str!("reduction.wgsl"), "main");
        remaining = remaining.div_ceil(128);
        device.execute(&groups, shader, [remaining as u32]);
        if remaining > 1 {
            buffer_n.write(&(remaining as u32));
            std::mem::swap(&mut buffer_a, &mut buffer_b);
        }
        count += 1;
    }
    let mut output: Vec<i32> = vec![0];
    pollster::block_on(if count % 2 == 0 {
        buffer_b.read(&mut output)
    } else {
        buffer_a.read(&mut output)
    })
    .expect("Failed to fetch data from output buffer");
    output[0]
}

fn main() {
    use std::time::Instant;

    // 128 needs to be subtracted as my GPU (GTX 1060) has a workgroup size limit of 65535.
    // 65535 * 128 = 8388480 = (2 << 22) - 128
    let data = generate_random_data((2 << 22) - 128);
    let now = Instant::now();
    let expected: i32 = data.iter().copied().reduce(|acc, e| acc + e).unwrap_or(0);
    println!("CPU Elapsed time: {:?}", now.elapsed());
    println!("Expected output: {:?}", expected);

    let now = Instant::now();
    let out = compute(&data);
    println!("Result: {:?}", out);
    println!("Elapsed time: {:?}", now.elapsed());
}
