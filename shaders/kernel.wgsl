@group(0) @binding(0)
var<storage, read_write> result: array<i32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    result[id.x] *= 2;
}



struct VertexOutput {
    @builtin(position) clip_position: vec4f, // 类似 gl_Position
};

// 顶点着色器
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4f(x, y, 0.0, 1.0);
    return out;
}

// 片元着色器
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(0.3, 0.2, 0.1, 1.0);
}