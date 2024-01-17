use std::{io::Write, iter, sync::Arc};

use app_surface::AppSurface;
use wgpu::{include_wgsl, util::DeviceExt, ShaderModule};

/// 定义了一个 Rust 结构体“ComputeInstance”和相应的原始表示“ComputeInstanceRaw”，以实现高效的内存处理。
///
/// Properties:
///
/// * `id`: “id”属性是一个无符号 32 位整数，表示计算实例的唯一标识符。
/// * `position`: “position”属性表示计算实例在 3D 空间中的位置。它的类型为 `glam::Vec3`，它是一个 3D 向量，包含三个表示位置的 x、y 和 z 坐标的
/// `f32` 值。
/// * `radius`: “radius”属性表示计算实例的大小或范围。它的类型为“f32”，这意味着它是一个单精度浮点数。
/// * `velocity`: “velocity”属性表示“ComputeInstance”移动的速度和方向。它是一个 `glam::Vec3`，它是一个 3 维向量，存储速度的 x、y 和 z 分量。
pub struct ComputeInstance {
    id: u32,
    position: glam::Vec3,
    radius: f32,
    velocity: glam::Vec3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct ComputeInstanceRaw {
    id: u32,
    radius: f32,
    _padding_radius: [u32; 2],
    position: [f32; 3],
    _padding_position: u32,
    velocity: [f32; 3],
    _padding_velocity: u32,
}

impl ComputeInstance {
    pub fn to_raw(&self) -> ComputeInstanceRaw {
        ComputeInstanceRaw {
            id: self.id,
            position: self.position.to_array(),
            radius: self.radius,
            velocity: self.velocity.to_array(),
            _padding_position: 0,
            _padding_radius: [0; 2],
            _padding_velocity: 0,
        }
    }
}

pub struct ComputeTestNode {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    buffer_len: u32,
    instances_buffer: Arc<wgpu::Buffer>,
    output_result_buffer: Arc<wgpu::Buffer>,
    output_cnt_buffer: Arc<wgpu::Buffer>,
}

impl ComputeTestNode {
    pub fn new(app: &AppSurface, shader: ShaderModule, buffer_len: u32) -> Self {
        let bind_group_layout =
            app.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Test Layout"),
                    entries: &[
                        // 一个只读的存储缓冲区，用于输入
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // 一个可读可写的存储缓冲区，用于输出结果
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // 一个可读可写的存储缓冲区，用于输出
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let pipeline_layout = app
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Test Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = app
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Test Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "naive_collision_test",
            });

        println!(
            "instance raw size: {}",
            std::mem::size_of::<ComputeInstanceRaw>()
        );

        let instances_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Input Buffer"),
            size: buffer_len as u64
                * std::mem::size_of::<ComputeInstanceRaw>() as wgpu::BufferAddress, // n * 32
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let output_result_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_len as u64
                // * buffer_len as u64
                * std::mem::size_of::<u32>() as wgpu::BufferAddress, // n * n * 4
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        let output_cnt_buffer = Arc::new(app.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Output Count Buffer"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            },
        ));

        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &instances_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_result_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_cnt_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            buffer_len,
            instances_buffer,
            output_result_buffer,
            output_cnt_buffer,
        }
    }
    pub fn dispatch<'a, 'b: 'a>(&'b self, cpass: &mut wgpu::ComputePass<'a>, workgroup_count: u32) {
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    pub fn write_input_buffer(&self, app: &AppSurface, instances: &[ComputeInstanceRaw]) {
        app.queue
            .write_buffer(&self.instances_buffer, 0, bytemuck::cast_slice(&instances));
    }
    pub fn read_output_buffer(&self, app: &AppSurface) {
        // let output_buffer = self.output_result_buffer.clone();

        // output_buffer
        //     .clone()
        //     .slice(..)
        //     .map_async(wgpu::MapMode::Read, move |result| {
        //         result.expect("failed to map storage buffer");
        //         let binding = output_buffer.clone();
        //         let contents = binding.slice(..).get_mapped_range();
        //         let readback = contents
        //             .chunks_exact(std::mem::size_of::<u32>())
        //             .map(|bytes| u32::from_ne_bytes(bytes.try_into().unwrap()))
        //             .collect::<Vec<_>>();
        //         println!("Output: {readback:?}");
        //     });

        let cnt_buffer = self.output_cnt_buffer.clone();

        cnt_buffer
        .clone()
        .slice(..)
        .map_async(wgpu::MapMode::Read,  move |result| {
            result.expect("failed to map storage buffer");
                let binding = cnt_buffer.clone();
                let contents = binding.slice(..).get_mapped_range();
                let readback = contents
                    .chunks_exact(std::mem::size_of::<u32>())
                    .map(|bytes| u32::from_ne_bytes(bytes.try_into().unwrap()))
                    .collect::<Vec<_>>();
                println!("Output: {readback:?}");
                // flush the stdout
                std::io::stdout().flush().unwrap();
            });
            
        // waiting for the map to complete
        while ! app.device.poll(wgpu::MaintainBase::Wait) {
            // println!("waiting for the map to complete");
        }
        
        let cnt_buffer2 = self.output_cnt_buffer.clone();
        {
            let bufferview = cnt_buffer2.slice(..).get_mapped_range();
            let readback = bufferview
                .chunks_exact(std::mem::size_of::<u32>())
                .map(|bytes| u32::from_ne_bytes(bytes.try_into().unwrap()))
                .collect::<Vec<_>>();
            println!("Output Count: {readback:?}");
        }

        // unmap the buffer
        cnt_buffer2.clone().unmap();

    }
}

pub fn do_compute(app: &AppSurface, points: &[glam::Vec3]) {
    let points_cnt = points.len() as u32;
    println!("points_cnt: {points_cnt}");

    let compute_shader = app
        .device
        .create_shader_module(include_wgsl!("../shaders/compute.wgsl"));

    let computer_node = ComputeTestNode::new(&app, compute_shader, points_cnt);

    let mut encoder = app
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

    let input_instances = points
        .iter()
        .enumerate()
        .map(|(i, point)| ComputeInstance {
            id: i as u32,
            position: *point,
            radius: 0.15,
            velocity: glam::Vec3::ZERO,
        })
        .map(|instance| instance.to_raw())
        .collect::<Vec<_>>();

    computer_node.write_input_buffer(&app, &input_instances);

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pass"),
            ..Default::default()
        });

        computer_node.dispatch(&mut cpass, points_cnt / 32 + 1);
    }

    // println!("input: {:?}", input_instances);
    
    app.queue.submit(iter::once(encoder.finish()));
    app.device.poll(wgpu::MaintainBase::Wait);
    
    computer_node.read_output_buffer(app);
    // wait for the compute shader to finish
}
