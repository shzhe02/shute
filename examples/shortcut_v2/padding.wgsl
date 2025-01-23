@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

struct Input {
    dim: u32,
    nn: u32
}

@group(0) @binding(2) var<uniform> input: Input;
@compute @workgroup_size(16, 16)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let i = global_id.x;
    let j = global_id.y;
    if (i >= dim || j >= dim) {
        return;
    }
    var smallest: u32 = 4294967294u;
    for (var k = 0u; k < dim; k += 1u) {
        let x = input[dim*j + k];
        let y = input[dim*k + i];
        let z = x + y;
        smallest = min(smallest, z);
    }
    output[dim*j + i] = smallest;
}
