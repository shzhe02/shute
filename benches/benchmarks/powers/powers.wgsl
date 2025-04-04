@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

struct Input {
    powers: u32
}

@group(0) @binding(2) var<uniform> params: Input;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    var sum: f32 = 0.0;
    var curr = input[i];
    for (var k = 1u; k <= params.powers; k += 1u) {
        sum += curr;
        curr *= input[i];
    }
    output[i] = sum;
}
