///#include "header.wgsl"


@group(0) @binding(0)
var<storage, read_write> params: Parameters;

@group(1) @binding(0)
var<storage, read_write> instances: array<Instance>;

@group(2) @binding(0)
var<storage, read_write> sort_params: SortParams;


// swap instance
fn swap(idx1 : u32, idx2 : u32) {
    var tmp = instances[idx1];
    instances[idx1] = instances[idx2];
    instances[idx2] = tmp;
}

fn agentlt(idx1 : u32, idx2 : u32) -> bool {
    return instances[idx1].cell_index < instances[idx2].cell_index;
}

fn agentgt(idx1 : u32, idx2 : u32) -> bool {
    return instances[idx1].cell_index > instances[idx2].cell_index;
}



@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id : vec3<u32>) {
    var global_tid = id.x;
    let j = sort_params.j;
    let k = sort_params.k;


    var l = global_tid ^ j; 
    if (l > global_tid) {
        if (  ((global_tid & k) == 0u && agentgt(global_tid, l)) || ((global_tid & k) != 0u && agentlt(global_tid, l))){
            swap(global_tid, l);
        }
    }
}
