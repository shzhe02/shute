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
        0,
        Some(bytemuck::cast_slice(&data).to_vec()),
    );
    let mut output_buffer = device.create_buffer(
        Some("output"),
        shute::BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        size_of_val(&data[..]) as u64,
        None,
    );
    device
        .execute(
            &mut vec![vec![&mut input_buffer, &mut output_buffer]],
            shader,
            (data.len() as u32, 1, 1),
        )
        .await;
    let output: Vec<u32> =
        bytemuck::cast_slice(&output_buffer.read_output_data().as_ref().unwrap()).to_vec();
    dbg!(output);
}

fn main() {
    pollster::block_on(test());
}
