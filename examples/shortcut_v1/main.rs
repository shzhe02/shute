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

async fn compute(data: &Vec<u32>, dim: u32) -> Vec<u32> {
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
        shute::BufferInit::WithData(dim),
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
// fn cpu_compute(data: &Vec<u32>, dim: u32) -> Vec<u32> {
//     let dim = dim as usize;
//     let mut output = vec![0; dim * dim];
//     for i in 0..dim {
//         for j in 0..dim {
//             let mut smallest = u32::MAX;
//             for k in 0..dim {
//                 let sum = data[dim * i + k] + data[dim * k + j];
//                 smallest = std::cmp::min(sum, smallest);
//             }
//             output[dim * i + j] = smallest;
//         }
//     }
//     output
// }

fn main() {
    use std::time::Instant;
    let dim = 4000u32;
    let data = generate_data(dim as usize);
    let now = Instant::now();
    pollster::block_on(compute(&data, dim));
    let gpu_elapsed = now.elapsed();
    println!("GPU took: {:.2?}", gpu_elapsed);
    // let now = Instant::now();
    // let cpu_result = cpu_compute(&data, dim);
    // let cpu_elapsed = now.elapsed();
    // println!(
    //     "GPU took: {:.2?}, CPU took: {:.2?}",
    //     gpu_elapsed, cpu_elapsed
    // );
    // println!("Verifying correctness...");
    // if cpu_result
    //     .iter()
    //     .zip(gpu_result.iter())
    //     .all(|(a, b)| a == b)
    // {
    //     println!("Results are correct.");
    // } else {
    //     println!("Results are inconsistent.");
    // }
}
