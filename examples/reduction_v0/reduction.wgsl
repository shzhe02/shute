@group(0) @binding(0) var<storage, read_write> input: array<i32>;
@group(0) @binding(1) var<storage, read_write> output: array<i32>;
@group(0) @binding(2) var<uniform> n: u32;

var<workgroup> shared_data: array<i32, 128>;

@compute @workgroup_size(128)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) wg_id: vec3<u32>
) {
    let t_id = local_id.x;
    let i = wg_id.x * 128 + local_id.x;
    if (i < n) {
        shared_data[t_id] = input[i];
    }
    workgroupBarrier();

    for (var s = 1u; s < 128u; s *= 2u) {
        if (t_id % (2u * s) == 0u) {
            shared_data[t_id] += shared_data[t_id + s];
        }
        workgroupBarrier();
    }

    if (t_id == 0u) {
        output[wg_id.x] = shared_data[0];
    }
}
