use rand::Rng;
use shute::{Buffer, Instance, PowerPreference};

fn generate_data(dim: u32) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    (0..dim * dim).map(|_| rng.gen::<u32>()).collect()
}

async fn compute() {
    let instance = Instance::new();
    let device = instance
        .autoselect(PowerPreference::HighPerformance)
        .await
        .unwrap();
    let dim = 3;
    // let data = generate_data(dim);
    let data = vec![0, 8, 2, 1, 0, 9, 4, 5, 0];
    // Correct output: 0, 7, 2, 1, 0, 3, 4, 5, 0
    for line in data.chunks(dim as usize) {
        println!("{:.2?}", line);
    }
    println!("=======");

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
    let shader = device.create_shader_module(include_str!("shortcut.wgsl"), "main".to_string());
    let mut groups: Vec<Vec<&mut Buffer>> =
        vec![vec![&mut input_buffer, &mut output_buffer, &mut dim_buffer]];
    device.execute(&mut groups, shader, (dim, dim, 1)).await;
    let output: Vec<u32> =
        bytemuck::cast_slice(&output_buffer.read_output_data().as_ref().unwrap()).to_vec();
    for line in output.chunks(dim as usize) {
        println!("{:.2?}", line);
    }
}

fn main() {
    pollster::block_on(compute());
}
