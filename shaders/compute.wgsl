// 这个是储存物体实例的 Buffer 的结构体的定义
// 我们假定所有的物体都是小球
// 所以只需要储存ID，位置，半径，速度
struct Instances {
    id: u32,
    radius: f32,
    position: vec3f,
    velocity: vec3f,
}

// 力的常数 K
const K: f32 = 10.0;

const G: f32 = 9.8;

// 时间步长
@group(0) @binding(0)
var<storage, read> time_step: f32;
// 边界，[-boundary, boundary]，如果距离 boundary 小于半径，就认为发生了碰撞
@group(0) @binding(1)
var<storage, read> boundary: f32;

// 输入，物体实例的 buffer
@group(0) @binding(2)
var<storage, read> instances: array<Instances>;
// 输出，碰撞检测的结果的数量
@group(0) @binding(3)
var<storage, read_write> collision_result_count: atomic<u32>;
// 输出，碰撞检测后的速度和位置
@group(0) @binding(4)
var<storage, read_write> result_position: array<vec3f>;
@group(0) @binding(5)
var<storage, read_write> result_velocity: array<vec3f>;

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

    var total_force = vec3f(0.0, 0.0, 0.0);
    for (var i = 0u; i < len; i = i + 1u) {

        if (i == my_idx) {
            continue;
        }
        let other_instance = instances[i];
        let rel_pos = my_instance.position - other_instance.position;
        let distance = length(rel_pos);
        let delta = -distance + my_instance.radius + other_instance.radius;
        
        if (delta > 0.0) {
            // 发生了碰撞
            atomicAdd(&collision_result_count, 1u);
            let normal = normalize(rel_pos);
            let f = K * delta * normal; // force
            total_force = total_force + f; // 累加所有的力
        }
    }

    // 计算加速度
    let mass = my_instance.radius * my_instance.radius * my_instance.radius;
    let acceleration = total_force / mass + vec3f(0.0, -G, 0.0);
    // 计算速度
    var velocity = my_instance.velocity + acceleration * time_step;
    
    // 和边界的碰撞
    // x 方向
    let delta_x_pos = my_instance.position.x + my_instance.radius - boundary;
    if(delta_x_pos > 0.0) {
        velocity.x = - abs(velocity.x);
    }
    let delta_x_neg = my_instance.position.x - my_instance.radius + boundary;
    if(delta_x_neg < 0.0) {
        velocity.x = abs(velocity.x);
    }
    // y 方向
    let delta_y_pos = my_instance.position.y + my_instance.radius - boundary;
    if(delta_y_pos > 0.0) {
        velocity.y = - abs(velocity.y);
    }
    let delta_y_neg = my_instance.position.y - my_instance.radius + boundary;
    if(delta_y_neg < 0.0) {
        velocity.y = abs(velocity.y);
    }
    // z 方向
    let delta_z_pos = my_instance.position.z + my_instance.radius - boundary;
    if(delta_z_pos > 0.0) {
        velocity.z = - abs(velocity.z);
    }
    let delta_z_neg = my_instance.position.z - my_instance.radius + boundary;
    if(delta_z_neg < 0.0) {
        velocity.z = abs(velocity.z);
    }    
    
    
    // 计算位置
    let position = my_instance.position + my_instance.velocity * time_step + acceleration * time_step * time_step * 0.5;
    // 将结果写入输出
    result_position[id.x] = position;
    result_velocity[id.x] = velocity * 0.99;
}