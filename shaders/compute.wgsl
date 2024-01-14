@group(0) @binding(0)
var<storage, read> input: array<i32>;
@group(0) @binding(1)
var<storage, read_write> output: array<i32>;

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    output[id.x] = input[id.x] * 2;
}