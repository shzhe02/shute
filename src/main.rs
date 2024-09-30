use shute::{load_wgsl_shader, ComputeDevice};

fn main() {
    // === Turn wgsl shader into spirv with Naga ===
    let spv_out = load_wgsl_shader!("shaders/doubler.wgsl");

    let mut compute_device = ComputeDevice::autoselect(&spv_out, "doubler");

    compute_device.add_buffer(0, 0..500u32);
    compute_device.add_buffer(1, 5..505u32);
    compute_device.add_buffer(2, 0..500u32);

    compute_device.execute();

    dbg!(compute_device.read_buffer(2));
}
