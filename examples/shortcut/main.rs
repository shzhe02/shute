use rand::Rng;
use shute::{Instance, PowerPreference};

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
        shute::BufferType::StorageBuffer,
        0,
        Some(bytemuck::cast_slice(&data).to_vec()),
        false,
    );
    let mut output_buffer = device.create_buffer(
        Some("output"),
        shute::BufferType::StorageBuffer,
        size_of_val(&data[..]) as u64,
        None,
        true,
    );
    let mut dim_buffer = device.create_buffer(Some("dim"), shute::BufferType::UniformBuffer, , , )
    
}

fn main() {
    pollster::block_on(compute());
}
