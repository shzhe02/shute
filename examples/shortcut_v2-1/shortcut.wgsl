@group(0) @binding(0) var<storage, read_write> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> input_t: array<u32>;
@group(0) @binding(2) var<storage, read_write> output: array<u32>;

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

    var v = array<array<u32, 8>, 8>();

    // for (var ib = 0u; ib < 8u; ib += 1u) {
    //     for (var jb = 0u; jb < 8u; jb += 1u) {
    //         v[ib][jb] = 4294967295u;
    //     }
    // }
    v[0][0] = 4294967295u;
    v[0][1] = 4294967295u;
    v[0][2] = 4294967295u;
    v[0][3] = 4294967295u;
    v[0][4] = 4294967295u;
    v[0][5] = 4294967295u;
    v[0][6] = 4294967295u;
    v[0][7] = 4294967295u;
    v[1][0] = 4294967295u;
    v[1][1] = 4294967295u;
    v[1][2] = 4294967295u;
    v[1][3] = 4294967295u;
    v[1][4] = 4294967295u;
    v[1][5] = 4294967295u;
    v[1][6] = 4294967295u;
    v[1][7] = 4294967295u;
    v[2][0] = 4294967295u;
    v[2][1] = 4294967295u;
    v[2][2] = 4294967295u;
    v[2][3] = 4294967295u;
    v[2][4] = 4294967295u;
    v[2][5] = 4294967295u;
    v[2][6] = 4294967295u;
    v[2][7] = 4294967295u;
    v[3][0] = 4294967295u;
    v[3][1] = 4294967295u;
    v[3][2] = 4294967295u;
    v[3][3] = 4294967295u;
    v[3][4] = 4294967295u;
    v[3][5] = 4294967295u;
    v[3][6] = 4294967295u;
    v[3][7] = 4294967295u;
    v[4][0] = 4294967295u;
    v[4][1] = 4294967295u;
    v[4][2] = 4294967295u;
    v[4][3] = 4294967295u;
    v[4][4] = 4294967295u;
    v[4][5] = 4294967295u;
    v[4][6] = 4294967295u;
    v[4][7] = 4294967295u;
    v[5][0] = 4294967295u;
    v[5][1] = 4294967295u;
    v[5][2] = 4294967295u;
    v[5][3] = 4294967295u;
    v[5][4] = 4294967295u;
    v[5][5] = 4294967295u;
    v[5][6] = 4294967295u;
    v[5][7] = 4294967295u;
    v[6][0] = 4294967295u;
    v[6][1] = 4294967295u;
    v[6][2] = 4294967295u;
    v[6][3] = 4294967295u;
    v[6][4] = 4294967295u;
    v[6][5] = 4294967295u;
    v[6][6] = 4294967295u;
    v[6][7] = 4294967295u;
    v[7][0] = 4294967295u;
    v[7][1] = 4294967295u;
    v[7][2] = 4294967295u;
    v[7][3] = 4294967295u;
    v[7][4] = 4294967295u;
    v[7][5] = 4294967295u;
    v[7][6] = 4294967295u;
    v[7][7] = 4294967295u;
    for (var k = 0u; k < dim; k += 1u) {
        var x = array<u32, 8>();
        var y = array<u32, 8>();
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
