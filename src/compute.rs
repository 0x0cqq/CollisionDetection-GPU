use std::{iter, sync::Arc};

use app_surface::AppSurface;

use crate::utils;

#[derive(Debug, Copy, Clone)]
pub struct ComputeInstance {
    pub id: u32,
    pub position: glam::Vec3,
    pub radius: f32,
    pub velocity: glam::Vec3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct ComputeInstanceRaw {
    id: u32,
    radius: f32,
    cell_index: u32,
    _padding_radius: u32,
    position: [f32; 3],
    _padding_position: u32,
    velocity: [f32; 3],
    _padding_velocity: u32,
}

impl ComputeInstance {
    pub fn to_raw(&self) -> ComputeInstanceRaw {
        ComputeInstanceRaw {
            id: self.id,
            cell_index: 0,
            position: self.position.to_array(),
            radius: self.radius,
            velocity: self.velocity.to_array(),
            _padding_position: 0,
            _padding_radius: 0,
            _padding_velocity: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Parameters {
    pub time_step: f32,
    pub boundary: f32,
    pub grid_size: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SortParams {
    pub j: u32,
    pub k: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CellIndex {
    pub start: u32,
    pub end: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Result {
    pub position: [f32; 3],
    _padding: u32,
    pub velocity: [f32; 3],
    _padding2: u32,
}

// 这里面不存 Buffer，负责逻辑部分
pub struct ComputeNode {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::ComputePipeline,
}

pub fn new_layout_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn new_group_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: None,
        }),
    }
}

impl ComputeNode {
    pub fn new(
        app: &AppSurface,
        shader_source: &str,
        buffers: &[Arc<wgpu::Buffer>],
        label: &str,
    ) -> Self {
        let header = include_str!("../shaders/header.wgsl");

        let full_shader_source =
            wgpu::ShaderSource::Wgsl(format!("{}\n{}", header, shader_source).into());

        let shader_module = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(format!("{} Shader", label).as_str()),
                source: full_shader_source,
            });

        // layout 都是统一的
        let bind_group_layout =
            app.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(format!("{} Bind Group Layout", label).as_str()),
                    entries: &[new_layout_entry(0, false)],
                });
        let pipeline_layout = app
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(format!("{} Pipeline Layout", label).as_str()),
                bind_group_layouts: &[
                    &bind_group_layout,
                    &bind_group_layout,
                    &bind_group_layout,
                    &bind_group_layout,
                    &bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let pipeline = app
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(format!("{} Pipeline", label).as_str()),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "main",
            });

        let mut bind_groups = Vec::new();

        for (i, buffer) in buffers.iter().enumerate() {
            let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(format!("{} Bind Group {}", label, i).as_str()),
                layout: &bind_group_layout,
                entries: &[new_group_entry(0, &buffer)],
            });
            bind_groups.push(bind_group);
        }

        Self {
            bind_group_layout,
            bind_groups,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn dispatch<'a, 'b: 'a>(&'b self, cpass: &mut wgpu::ComputePass<'a>, workgroup_count: u32) {
        cpass.set_pipeline(&self.pipeline);
        // scan over vec
        for (i, bind_group) in self.bind_groups.iter().enumerate() {
            cpass.set_bind_group(i as u32, bind_group, &[]);
        }
        cpass.dispatch_workgroups(workgroup_count, 1, 1);
    }
}

pub fn read_buffer_bytes(app: &AppSurface, buffer: Arc<wgpu::Buffer>) -> Vec<u8> {
    buffer
        .clone()
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            result.expect("failed to map storage buffer");
        });

    while !app.device.poll(wgpu::MaintainBase::Wait) {
        // println!("waiting for the map to complete");
    }

    let mut results: Vec<u8> = Vec::new();

    {
        // map the buffer and read results
        let bufferview = buffer.slice(..).get_mapped_range();
        let readback = bufferview.to_vec();
        // copy readback to results
        results.extend_from_slice(&readback);
    }

    // unmap the buffer
    buffer.clone().unmap();

    results
}

pub struct ComputeState {
    pub instances: Vec<ComputeInstance>,
    buffer_len: u32,                           // the number of instances
    boundary: f32,                             // the boundary of the simulation
    pub params_buffer: Arc<wgpu::Buffer>,      // group 0
    pub instances_buffer: Arc<wgpu::Buffer>,   // group 1
    pub sort_params_buffer: Arc<wgpu::Buffer>, // group 2
    pub cell_index_buffer: Arc<wgpu::Buffer>,  // group 3
    pub result_buffer: Arc<wgpu::Buffer>,      // group 4

    pub assign_cell_node: ComputeNode,
    pub sort_node: ComputeNode,
    pub build_grid_node: ComputeNode,
    pub collision_node: ComputeNode,
}

impl ComputeState {
    pub fn new(app: &AppSurface, buffer_len: u32, boundary: f32) -> Self {
        // 创建 buffer
        let params_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Params Buffer"),
            size: std::mem::size_of::<Parameters>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let instances_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instances Buffer"),
            size: std::mem::size_of::<ComputeInstanceRaw>() as u64 * buffer_len as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let sort_params_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sort Params Buffer"),
            size: std::mem::size_of::<SortParams>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let cell_index_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cell Index Buffer"),
            size: std::mem::size_of::<CellIndex>() as u64 * buffer_len as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let result_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Position Buffer"),
            size: std::mem::size_of::<Result>() as u64 * buffer_len as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        let buffers = vec![
            params_buffer.clone(),
            instances_buffer.clone(),
            sort_params_buffer.clone(),
            cell_index_buffer.clone(),
            result_buffer.clone(),
        ];

        // 创建 compute node

        let assign_cell_node = ComputeNode::new(
            app,
            include_str!("../shaders/assign.wgsl"),
            &buffers,
            "Assign Cell",
        );
        let sort_node =
            ComputeNode::new(app, include_str!("../shaders/sort.wgsl"), &buffers, "Sort");
        let build_grid_node = ComputeNode::new(
            app,
            include_str!("../shaders/build_grid.wgsl"),
            &buffers,
            "Build Grid",
        );
        let collision_node = ComputeNode::new(
            app,
            include_str!("../shaders/collision.wgsl"),
            &buffers,
            "Collision",
        );

        Self {
            instances: Vec::new(),
            buffer_len,
            boundary,
            params_buffer,
            instances_buffer,
            sort_params_buffer,
            cell_index_buffer,
            result_buffer,
            assign_cell_node,
            sort_node,
            build_grid_node,
            collision_node,
        }
    }

    // 将 CPU 中的 Instance 数据写到 GPU 的 Instance Buffer 中
    pub fn write_instances_buffer(&self, app: &AppSurface, instances: &[ComputeInstance]) {
        app.queue.write_buffer(
            &self.instances_buffer,
            0,
            bytemuck::cast_slice(
                &instances
                    .iter()
                    .map(ComputeInstance::to_raw)
                    .collect::<Vec<_>>(),
            ),
        );
    }

    pub fn do_compute(&self, app: &AppSurface, simulation_rounds: u32) {
        let mut encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute pass"),
                ..Default::default()
            });
            for _ in 0..simulation_rounds {
                // 以下是一次完整的碰撞检测,我们会切碎时间块之后再进行碰撞检测
                // assign cell
                // self.assign_cell_node.dispatch(&mut cpass, 128);

                // // bitonic sort
                // // adapted from Wikipedia's non-recursive example of bitonic sort:
                // // https://en.wikipedia.org/wiki/Bitonic_sorter
                // let mut k = 2;
                // while k <= self.buffer_len {
                //     // k is doubled every iteration
                //     let mut j = k >> 1;
                //     while j > 0 {
                //         // j is halved at every iteration, with truncation of fractional parts
                //         let sort_params = SortParams { j, k };
                //         app.queue.write_buffer(
                //             &self.sort_params_buffer,
                //             0,
                //             bytemuck::cast_slice(&[sort_params]),
                //         );

                //         self.sort_node.dispatch(&mut cpass, 128);

                //         j >>= 1;
                //     }
                //     k <<= 1;
                // }

                // // build grid
                // self.build_grid_node.dispatch(&mut cpass, 128);

                // collision detection
                self.collision_node
                    .dispatch(&mut cpass, self.buffer_len as u32 / 64 + 1);
            }
        }

        app.queue.submit(iter::once(encoder.finish()));
        app.device.poll(wgpu::MaintainBase::Wait);
    }

    pub fn update(&mut self, app: &AppSurface, dt: std::time::Duration) {
        let simulation_rounds = 10;

        // 首先把 instance buffer 写入 GPU
        self.write_instances_buffer(app, &self.instances);

        // 其次, params 也是每次不变的, 写入
        let params = Parameters {
            time_step: dt.as_secs_f32() / simulation_rounds as f32,
            boundary: self.boundary,
            grid_size: 1, // to be modified
        };

        app.queue.write_buffer(
            &self.params_buffer,
            0,
            bytemuck::cast_slice(&[params.clone()]),
        );

        // 执行计算
        self.do_compute(app, simulation_rounds);

        // 从 result 中把结果 readback 回来, 更新 instance, 注意 compute instance 在 CPU 里面是有序的
        let mapped_result = read_buffer_bytes(app, self.result_buffer.clone());

        let results: Vec<f32> = utils::bytes_to_f32(&mapped_result);

        for i in 0..self.buffer_len {
            // 一个 result 有 8 个 f32, 只有六个是有用的
            let pos = [
                results[i as usize * 8],
                results[i as usize * 8 + 1],
                results[i as usize * 8 + 2],
            ];
            self.instances[i as usize].position = glam::Vec3::from_array(pos);

            let vel = [
                results[i as usize * 8 + 4],
                results[i as usize * 8 + 5],
                results[i as usize * 8 + 6],
            ];

            self.instances[i as usize].velocity = glam::Vec3::from_array(vel);
        }
    }
}
