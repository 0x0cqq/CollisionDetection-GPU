use std::{iter, sync::Arc};

use app_surface::AppSurface;
use wgpu::{util::DeviceExt, ShaderModule};

use crate::utils;

/// 定义了一个 Rust 结构体“ComputeInstance”和相应的原始表示“ComputeInstanceRaw”，以实现高效的内存处理。
///
/// Properties:
///
/// * `id`: “id”属性是一个无符号 32 位整数，表示计算实例的唯一标识符。
/// * `position`: “position”属性表示计算实例在 3D 空间中的位置。它的类型为 `glam::Vec3`，它是一个 3D 向量，包含三个表示位置的 x、y 和 z 坐标的
/// `f32` 值。
/// * `radius`: “radius”属性表示计算实例的大小或范围。它的类型为“f32”，这意味着它是一个单精度浮点数。
/// * `velocity`: “velocity”属性表示“ComputeInstance”移动的速度和方向。它是一个 `glam::Vec3`，它是一个 3 维向量，存储速度的 x、y 和 z 分量。
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

pub struct ComputeState {
    pub instances: Vec<ComputeInstance>,
}

pub struct ComputeTestNode {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    buffer_len: u32,
    instances_buffer: Arc<wgpu::Buffer>,
    time_step_buffer: Arc<wgpu::Buffer>,
    boundary_buffer: Arc<wgpu::Buffer>,
    output_cnt_buffer: Arc<wgpu::Buffer>,
    output_position_buffer: Arc<wgpu::Buffer>,
    output_velocity_buffer: Arc<wgpu::Buffer>,
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
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
            "Compute instance raw size: {}",
            std::mem::size_of::<ComputeInstanceRaw>()
        );

        let time_step_buffer = Arc::new(app.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Time Step Buffer"),
                contents: bytemuck::cast_slice(&[0.05f32]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            },
        ));

        let boundary_buffer = Arc::new(app.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Boundary Buffer"),
                contents: bytemuck::cast_slice(&[10.0f32]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            },
        ));

        let instances_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Input Buffer"),
            size: buffer_len as u64
                * std::mem::size_of::<ComputeInstanceRaw>() as wgpu::BufferAddress, // n * 32
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let output_cnt_buffer = Arc::new(app.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Output Count Buffer"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            },
        ));

        let output_velocity_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Velocity Buffer"),
            size: buffer_len as u64 * 4 * std::mem::size_of::<u32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        let output_position_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Position Buffer"),
            size: buffer_len as u64 * 4 * std::mem::size_of::<u32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &time_step_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &boundary_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &instances_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_cnt_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_position_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_velocity_buffer,
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
            time_step_buffer,
            boundary_buffer,
            instances_buffer,
            output_cnt_buffer,
            output_position_buffer,
            output_velocity_buffer,
        }
    }
    pub fn dispatch<'a, 'b: 'a>(&'b self, cpass: &mut wgpu::ComputePass<'a>, workgroup_count: u32) {
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.dispatch_workgroups(workgroup_count, 1, 1);
    }

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

    pub fn read_buffer_bytes(&self, app: &AppSurface, buffer: Arc<wgpu::Buffer>) -> Vec<u8> {
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

    pub fn output_buffer(&self, app: &AppSurface) {
        // cnt buffer
        utils::output_bytes_as_u32(
            &self.read_buffer_bytes(app, self.output_cnt_buffer.clone()),
            "Count",
        );
        // position buffer
        utils::output_bytes_as_f32(
            &self.read_buffer_bytes(app, self.output_position_buffer.clone()),
            "Position",
        );
        // velocity buffer
        utils::output_bytes_as_f32(
            &self.read_buffer_bytes(app, self.output_velocity_buffer.clone()),
            "Velocity",
        );
    }

    pub fn read_velocity_buffer(&self, app: &AppSurface) -> Vec<glam::Vec3> {
        let bytes = self.read_buffer_bytes(app, self.output_velocity_buffer.clone());

        let result_raw = utils::bytes_to_f32(&bytes);

        let mut result: Vec<glam::Vec3> = Vec::new();

        for i in 0..result_raw.len() / 4 {
            result.push(glam::Vec3::new(
                result_raw[i * 4],
                result_raw[i * 4 + 1],
                result_raw[i * 4 + 2],
            ));
        }

        result
    }

    pub fn read_position_buffer(&self, app: &AppSurface) -> Vec<glam::Vec3> {
        let bytes = self.read_buffer_bytes(app, self.output_position_buffer.clone());

        let result_raw = utils::bytes_to_f32(&bytes);

        let mut result: Vec<glam::Vec3> = Vec::new();

        for i in 0..result_raw.len() / 4 {
            result.push(glam::Vec3::new(
                result_raw[i * 4],
                result_raw[i * 4 + 1],
                result_raw[i * 4 + 2],
            ));
        }

        result
    }

    pub fn set_boundary(&self, app: &AppSurface, boundary: f32) {
        app.queue
            .write_buffer(&self.boundary_buffer, 0, bytemuck::cast_slice(&[boundary]));
    }

    pub fn set_time_step(&self, app: &AppSurface, time_step: f32) {
        app.queue.write_buffer(
            &self.time_step_buffer,
            0,
            bytemuck::cast_slice(&[time_step]),
        );
    }

    pub fn collision_test_cpu(instances: &[ComputeInstance]) {
        let start = std::time::Instant::now();

        let mut ans = 0;
        let len = instances.len();

        for i in 0..len {
            for j in 0..len {
                if i == j {
                    continue;
                }
                let instance_i = &instances[i];
                let instance_j = &instances[j];
                let distance = (instance_i.position - instance_j.position).length();
                if distance < instance_i.radius + instance_j.radius {
                    ans += 1;
                }
            }
        }

        let end = std::time::Instant::now();

        println!("Compute time (CPU): {:?}, ans: {}", end - start, ans);
    }
}

pub fn do_compute(app: &AppSurface, compute_node: &ComputeTestNode) {
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

        compute_node.dispatch(&mut cpass, compute_node.buffer_len / 64 + 1);
    }

    app.queue.submit(iter::once(encoder.finish()));
    app.device.poll(wgpu::MaintainBase::Wait);
}
