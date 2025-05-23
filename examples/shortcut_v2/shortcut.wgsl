@group(0) @binding(0) var<storage, read_write> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> input_t: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

struct Input {
    dim: u32,
    nn: u32
}

@group(0) @binding(3) var<uniform> params: Input;
@compute @workgroup_size(8, 8)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let ia = local_id.x;
    let ja = local_id.y;
    let ic = workgroup_id.x;
    let jc = workgroup_id.y;
    let nn = params.nn;
    let dim = params.dim;

    var v = array<array<f32, 8>, 8>();

    for (var ib = 0u; ib < 8u; ib += 1u) {
        for (var jb = 0u; jb < 8u; jb += 1u) {
            v[ib][jb] = 10.0;
        }
    }
    for (var k = 0u; k < dim; k += 1u) {
        var x = array<f32, 8>();
        var y = array<f32, 8>();
        for (var ib = 0u; ib < 8u; ib += 1u) {
            let i = ic * 64u + ib * 8u + ia;
            x[ib] = input_t[nn*k + i];
        }
        for (var jb = 0u; jb < 8u; jb += 1u) {
            let j = jc * 64u + jb * 8u + ja;
            y[jb] = input[nn*k + j];
        }
        for (var ib = 0u; ib < 8u; ib += 1u) {
            for (var jb = 0u; jb < 8u; jb += 1u) {
                v[ib][jb] = min(v[ib][jb], x[ib] + y[jb]);
            }
        }
    }
    for (var ib = 0u; ib < 8u; ib += 1u) {
        for (var jb = 0u; jb < 8u; jb += 1u) {
            let i = ic * 64u + ib * 8u + ia;
            let j = jc * 64u + jb * 8u + ja;
            if (i < dim && j < dim) {
                output[dim*i + j] = v[ib][jb];
            }
        }
    }
}
