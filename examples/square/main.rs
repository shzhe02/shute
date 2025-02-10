use shute::{Instance, PowerPreference};

fn compute(data: &mut Vec<u32>) {
    let instance = Instance::new();
    let device = pollster::block_on(
        instance.autoselect(PowerPreference::HighPerformance, shute::LimitType::Highest),
    )
    .unwrap();
    let shader = device.create_shader_module(include_str!("square.wgsl"), "main");
    let mut input_buffer = device.create_buffer(
        Some("input"),
        shute::BufferType::StorageBuffer {
            output: false,
            read_only: true,
        },
        shute::BufferInit::WithData(&data),
    );
    let size = data.len();
    let mut output_buffer = device.create_buffer(
        Some("output"),
        shute::BufferType::StorageBuffer {
            output: true,
            read_only: false,
        },
        shute::BufferInit::<u32>::WithSize(size),
    );
    let groups = vec![vec![&mut input_buffer, &mut output_buffer]];
    device.execute_blocking(&groups, shader, [size as u32]);
    pollster::block_on(output_buffer.fetch_data_from_device(data));
}

fn main() {
    let mut data: Vec<u32> = (0..200).collect();
    for line in data.chunks(10) {
        println!("{:?}", line);
    }
    compute(&mut data);
    for line in data.chunks(10) {
        println!("{:?}", line);
    }
}
