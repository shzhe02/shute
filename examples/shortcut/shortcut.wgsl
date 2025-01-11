@group(0) @binding(0) var<storage, read_write> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

struct Dim {
    dim: u32
}

@group(0) @binding(2) var<uniform>  dim: Dim;

@compute @workgroup_size(16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    let j = global_id.y;
    if (i >= dim.dim || j >= dim.dim) {
        return;
    }
    var smallest: f32 = bitcast<f32>(0x7f7fffff);
    for (var k = 0; k < n; k += 1) {
        let x = input[n*i + k];
        let y = input[n*k + j];
        let z = x + y;
        smallest = min(max_f32, z);
    }
    output[n*i + j] = smallest;
}
