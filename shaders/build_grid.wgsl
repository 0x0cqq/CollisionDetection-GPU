///#include "header.wgsl"

@group(0) @binding(0)
var<storage, read_write> params: Parameters;

@group(1) @binding(0)
var<storage, read_write> instances: array<Instance>;

@group(3) @binding(0)
var<storage, read_write> cells: array<CellIndex>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {    
    let idx = id.x;

    if (idx >= arrayLength(&instances)) {
        return;
    }

    let inst = instances[idx];

    if(idx == 0u) {
        cells[inst.cell_index].start = idx;
        return;
    }

    let p_inst = instances[idx - 1u];
    if(idx == arrayLength(&instances) - 1u) {
        cells[inst.cell_index].end = idx + 1u;
    }

    if (inst.cell_index != p_inst.cell_index) {
        cells[inst.cell_index].start = idx;
        cells[p_inst.cell_index].end = idx;
    }
}