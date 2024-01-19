///#include "header.wgsl"

@group(0) @binding(0)
var<storage, read_write> params: Parameters;

@group(3) @binding(0)
var<storage, read_write> cells: array<CellIndex>;

// memset the cells' start and end indices to 0
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>,  @builtin(num_workgroups) num_groups: vec3<u32>) {
    let total_cells_count = arrayLength(&cells);
    let workgroup_size = 256u;
    let num_threads = num_groups.x * workgroup_size;

    for(var base = 0u; base < total_cells_count; base = base + num_threads) {
        let my_idx = base + id.x;
        if (my_idx >= total_cells_count) {
            break;
        }

        cells[my_idx].start = 0u;
        cells[my_idx].end = 0u;
    }
}