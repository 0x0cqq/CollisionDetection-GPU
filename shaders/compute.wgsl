// 这个是储存物体实例的 Buffer 的结构体的定义
// 我们假定所有的物体都是小球
// 所以只需要储存ID，位置，半径，速度
struct Instances {
    id: u32,
    radius: f32,
    position: vec3f,
    velocity: vec3f,
}

// 输入，物体实例的 buffer
@group(0) @binding(0)
var<storage, read> instances: array<Instances>;
// 输出，碰撞检测的结果
@group(0) @binding(1)
var<storage, read_write> collision_result: array<u32>;
// 输出，碰撞检测的结果的数量
@group(0) @binding(2)
var<storage, read_write> collision_result_count: atomic<u32>;

@compute @workgroup_size(32)
fn naive_collision_test(@builtin(global_invocation_id) id: vec3<u32>) {
    // 这是一个十分简单的实现的碰撞检测的方法
    // 每一个线程都会检测一个物体和其他所有物体之间的碰撞


    // 暂时不考虑速度，如果距离小于两个物体的半径之和，就认为发生了碰撞，将结果写入输出
    let my_instance = instances[id.x];
    let my_idx = id.x;
    let len = arrayLength(&instances);
    if (my_idx >= len) {
        return;        
    }
    for (var i = 0u; i < len; i = i + 1u) {
        if (i == my_idx) {
            continue;
        }
        let other_instance = instances[i];
        let distance = length(my_instance.position - other_instance.position);
        let result_id = my_idx * len + i;
        if (distance < my_instance.radius + other_instance.radius) {
            // collision_result[result_id] = 1u;
            atomicAdd(&collision_result_count, 1u);
        } else {
            // collision_result[result_id] = 0u;
        }
    }
}