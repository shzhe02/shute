use shute::{load_wgsl_shader, ComputeDevice};

fn main() {
    // === Turn wgsl shader into spirv with Naga ===
    let mut compute_device = ComputeDevice::autoselect(
        &load_wgsl_shader!("./examples/doubler/shaders/doubler.wgsl"),
        "doubler",
    );

    compute_device.add_buffer(0, 0..500u32);
    compute_device.add_buffer(1, 5..505u32);
    compute_device.add_buffer(2, 0..500u32);

    compute_device.execute([500, 1, 1]);

    dbg!(compute_device.read_buffer(2));
}
