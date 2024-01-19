///#include "header.wgsl"

@group(0) @binding(0)
var<storage, read_write> params: Parameters;
    
@group(1) @binding(0)
var<storage, read_write> instances: array<Instance>;

@group(3) @binding(0)
var<storage, read_write> cells: array<CellIndex>;

@group(4) @binding(0)
var<storage, read_write> results: array<Result>;



@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let boundary = params.boundary;
    let time_step = params.time_step;

    // 暂时不考虑速度，如果距离小于两个物体的半径之和，就认为发生了碰撞，将结果写入输出
    let my_idx = id.x;
    let len = arrayLength(&instances);
    if (my_idx >= len) {
        return;        
    }
    let my_instance = instances[my_idx];

    let mass = my_instance.radius * my_instance.radius * my_instance.radius;
    var total_force = vec3f(0.0, 0.0, 0.0);
    for (var i = 0u; i < len; i = i + 1u) {
        if (i == my_idx) {
            continue;
        }
        let other_instance = instances[i];
        let rel_pos = my_instance.position - other_instance.position;
        let distance = length(rel_pos);
        let delta = -distance + my_instance.radius + other_instance.radius;
        
        if (delta > 0.0) {                      // 碰撞
            let normal = normalize(rel_pos);    // 碰撞法线
            let f = K * delta * normal;         // 碰撞力
            total_force = total_force + f;      // 累加所有的力
        }
    }

    let acceleration = total_force + vec3f(0.0, -G, 0.0);        // 加速度
    var velocity = my_instance.velocity + acceleration * time_step;     // 速度

    // 和边界的碰撞
    // x 方向
    let delta_x_pos = my_instance.position.x + my_instance.radius - boundary;
    if(delta_x_pos > 0.0) {     // 正方向
        velocity.x = - abs(velocity.x);
    }
    let delta_x_neg = my_instance.position.x - my_instance.radius + boundary;
    if(delta_x_neg < 0.0) {     // 负方向
        velocity.x = abs(velocity.x);
    }
    // y 方向
    let delta_y_pos = my_instance.position.y + my_instance.radius - boundary;
    if(delta_y_pos > 0.0) {     // 正方向
        velocity.y = - abs(velocity.y);
    }
    let delta_y_neg = my_instance.position.y - my_instance.radius + boundary;
    if(delta_y_neg < 0.0) {     // 负方向
        velocity.y = abs(velocity.y);
    }
    // z 方向
    let delta_z_pos = my_instance.position.z + my_instance.radius - boundary;
    if(delta_z_pos > 0.0) {     // 正方向
        velocity.z = - abs(velocity.z);
    }
    let delta_z_neg = my_instance.position.z - my_instance.radius + boundary;
    if(delta_z_neg < 0.0) {     // 负方向
        velocity.z = abs(velocity.z);
    }    
    
    
    // 计算位置
    let position = my_instance.position + my_instance.velocity * time_step + acceleration * time_step * time_step * 0.5;

    let inst_id = instances[my_idx].id;

    // 将结果写入输出
    results[inst_id].position = position;
    let v_len = length(velocity);
    results[inst_id].velocity = velocity * (1.0 - AR * v_len * v_len * v_len * time_step);
    // 也写回到输入，为了连续模拟
    instances[id.x].position = results[inst_id].position;
    instances[id.x].velocity = results[inst_id].velocity;
}