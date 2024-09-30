@group(0) @binding(0) var<storage, read> inputA: array<i32>;
@group(0) @binding(1) var<storage, read> inputB: array<i32>;
@group(0) @binding(2) var<storage, read_write> output: array<i32>;

@compute @workgroup_size(1)
fn doubler(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index: u32 = global_id.x;
    let sum = inputA[index] + inputB[index];
    output[index] = sum * 2i;
}
