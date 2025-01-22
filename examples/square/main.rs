use shute::{Instance, PowerPreference};

async fn test() {
    let instance = Instance::new();
    let device = instance
        .autoselect(PowerPreference::HighPerformance)
        .await
        .unwrap();
    let shader = device.create_shader_module(include_str!("square.wgsl"), "main".to_string());
    let data = (0..10).collect::<Vec<u32>>().to_vec();
    let mut input_buffer = device.create_buffer(
        Some("input"),
        shute::BufferType::StorageBuffer {
            output: false,
            read_only: true,
        },
        shute::BufferInit::WithData(data),
    );
    let size = input_buffer.size();
    let mut output_buffer = device.create_buffer(
        Some("output"),
        shute::BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        shute::BufferInit::WithSize(size),
    );
    device
        .execute(
            &mut vec![vec![&mut input_buffer, &mut output_buffer]],
            shader,
            (size, 1, 1),
        )
        .await;
    let output: Vec<u32> =
        bytemuck::cast_slice(&output_buffer.read_output_data().as_ref().unwrap()).to_vec();
    dbg!(output);
}

fn main() {
    pollster::block_on(test());
}
