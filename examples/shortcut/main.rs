use rand::Rng;
use shute::{Buffer, Instance, PowerPreference, ShaderType};

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
    let shader = device.create_shader_module("shortcut.wgsl", "main".to_string());
    let data = generate_data(32);

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
        shute::BufferInit::WithSize(input_buffer.size()),
    );
    let mut dim_buffer = device.create_buffer(
        Some("dim"),
        shute::BufferType::UniformBuffer,
        shute::BufferInit::WithData(32u32),
    );
    let shader = device.create_shader_module("shortcut.wgsl", "main".to_string());
    let mut groups: Vec<Vec<&mut Buffer<&dyn ShaderType>>> =
        vec![vec![&mut input_buffer, &mut output_buffer, &mut dim_buffer]];
    device.execute(&mut groups, shader, (32, 32, 1));
}

fn main() {
    pollster::block_on(compute());
}
