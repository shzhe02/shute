use rand::Rng;
use shute::{Buffer, Instance, PowerPreference};

fn generate_data(dim: u32) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim * dim).map(|_| rng.gen::<f32>()).collect()
}

async fn compute() {
    let instance = Instance::new();
    let device = instance
        .autoselect(PowerPreference::HighPerformance)
        .await
        .unwrap();
    let dim = 32;
    let data = generate_data(dim);
    for line in data.chunks(32) {
        println!("{:.2?}", line);
    }

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
    device.execute(&mut groups, shader, (32, 32, 1));
}

fn main() {
    pollster::block_on(compute());
}
