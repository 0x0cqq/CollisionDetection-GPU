///#include "header.wgsl"

@group(0) @binding(0)
var<storage, read_write> params: Parameters;

@group(1) @binding(0)
var<storage, read_write> instances: array<Instance>;

fn calculate_grid(position: vec3f) -> vec3u{
    // 距离原点的偏移
    let offset = position + vec3f(-params.boundary, -params.boundary, -params.boundary);
    // 网格的索引
    let grid_index = vec3u(
        u32(offset.x / params.grid_size),
        u32(offset.y / params.grid_size),
        u32(offset.z / params.grid_size)
    );
    return grid_index;
}


fn get_index_from_grid(grid_index: vec3u) -> u32 {
    let grid_count_x = u32(ceil(params.boundary * 2.0 / params.grid_size) + 0.5);
    let grid_count_y = u32(ceil(params.boundary * 2.0 / params.grid_size) + 0.5);
    let grid_count_z = u32(ceil(params.boundary * 2.0 / params.grid_size) + 0.5);
    return grid_index.x + grid_index.y * grid_count_x + grid_index.z * grid_count_x * grid_count_y;
}

fn get_grid_from_index(index: u32) -> vec3u {
    let grid_count_x = u32(ceil(params.boundary * 2.0 / params.grid_size) + 0.5);
    let grid_count_y = u32(ceil(params.boundary * 2.0 / params.grid_size) + 0.5);
    let grid_count_z = u32(ceil(params.boundary * 2.0 / params.grid_size) + 0.5);
    let z = index / (grid_count_x * grid_count_y);
    let y = (index - z * grid_count_x * grid_count_y) / grid_count_x;
    let x = index - z * grid_count_x * grid_count_y - y * grid_count_x;
    return vec3u(x, y, z);
}


// 将对应的格子数值放到 cell 中
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>,  @builtin(num_workgroups) num_groups: vec3<u32>) {
    let total_instance_count = arrayLength(&instances);
    let workgroup_size = 64u;
    let num_threads = num_groups.x * workgroup_size;

    for(var base = 0u; base < total_instance_count; base = base + num_threads) {
        let my_idx = base + id.x;
        if (my_idx >= total_instance_count) {
            break;
        }
        let grid = calculate_grid(instances[my_idx].position);
        let my_cell_index = get_index_from_grid(grid);
        instances[my_idx].cell_index = my_cell_index;
    }
}