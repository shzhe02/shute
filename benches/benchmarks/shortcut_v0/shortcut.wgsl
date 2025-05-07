@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@group(0) @binding(2) var<uniform> dim: u32;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    let j = global_id.y;
    if (i >= dim || j >= dim) {
        return;
    }
    var smallest: f32 = 10.0;
    for (var k = 0u; k < dim; k += 1u) {
        let x = input[dim*i + k];
        let y = input[dim*k + j];
        let z = x + y;
        smallest = min(smallest, z);
    }
    output[dim*i + j] = smallest;
}
