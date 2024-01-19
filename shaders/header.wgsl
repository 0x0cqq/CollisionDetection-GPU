// 这个是储存物体实例的 Buffer 的结构体的定义
// 我们假定所有的物体都是小球
// 所以只需要储存ID，位置，半径，速度
struct Instance {
    id: u32,
    radius: f32,
    cell_index: u32,
    // padding 4 bytes
    position: vec3f,
    velocity: vec3f,
}

struct Result {
    position: vec3f,
    // padding 4 bytes
    velocity: vec3f,
    // padding 4 bytes
}

struct Parameters {
    // 时间步长，单位是秒
    time_step: f32,
    // 边界，[-boundary, boundary]，如果距离 boundary 的平面小于半径，就认为发生了碰撞
    boundary: f32,
    // 从 -boundary 到 boundary 的格子大小，注意总共有三维
    grid_size: f32, 
}

// 双调排序的参数
struct SortParams {
    j: u32, 
    k: u32,
}

struct CellIndex {
    start: u32,
    end: u32,
}



// 力的常数 K
const K: f32 = 1000.0;

// 重力加速度
const G: f32 = 9.8;

// 碰撞能量损失
const E: f32 = 0.85;

// 空气阻力
const AR: f32 = 0.01;