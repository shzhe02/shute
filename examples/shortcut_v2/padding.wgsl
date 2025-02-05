@group(0) @binding(0) var<storage, read_write> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> input_t: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

struct Input {
    dim: u32,
    nn: u32
}

@group(0) @binding(3) var<uniform> params: Input;
@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let ja = local_id.x;
    let i = workgroup_id.y;
    
    for (var jb = 0u; jb < params.nn; jb += 64u) {
        let j = jb + ja;
        var v = 10.0;
        if (i < params.dim && j < params.dim) {
            v = output[params.dim * i + j];
        }
        input[params.nn*i + j] = v;
        input_t[params.nn*j + i] = v;
    }
}
