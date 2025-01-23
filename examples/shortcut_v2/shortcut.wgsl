@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

struct Input {
    dim: u32,
    nn: u32
}

@group(0) @binding(2) var<uniform> input: Input;
@compute @workgroup_size(8, 8)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let ia = local_id.x;
    let ja = local_id.y;
    let ic = workgroup_id.x;
    let jc = workgroup_id.y;

    let transposed = input + nn * nn;
    
    let v = array<array<u32>>;
    for (int ib = 0; ib < 8; ++ib) {
        for (int jb = 0; jb < 8; ++jb) {
            v[ib][jb] = HUGE_VALF;
        }
    }
    for (int k = 0; k < n; ++k) {
        float x[8];
        float y[8];
        for (int ib = 0; ib < 8; ++ib) {
            int i = ic * 64 + ib * 8 + ia;
            x[ib] = t[nn*k + i];
        }
        for (int jb = 0; jb < 8; ++jb) {
            int j = jc * 64 + jb * 8 + ja;
            y[jb] = d[nn*k + j];
        }
        for (int ib = 0; ib < 8; ++ib) {
            for (int jb = 0; jb < 8; ++jb) {
                v[ib][jb] = min(v[ib][jb], x[ib] + y[jb]);
            }
        }
    }
    for (int ib = 0; ib < 8; ++ib) {
        for (int jb = 0; jb < 8; ++jb) {
            int i = ic * 64 + ib * 8 + ia;
            int j = jc * 64 + jb * 8 + ja;
            if (i < n && j < n) {
                r[n*i + j] = v[ib][jb];
            }
        }
    }
    
    // let j = global_id.y;
    // if (i >= dim || j >= dim) {
    //     return;
    // }
    // var smallest: u32 = 4294967294u;
    // for (var k = 0u; k < dim; k += 1u) {
    //     let x = input[dim*j + k];
    //     let y = input[dim*k + i];
    //     let z = x + y;
    //     smallest = min(smallest, z);
    // }
    // output[dim*j + i] = smallest;
}
